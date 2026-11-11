//! Capa HTTP con el mismo contrato que la versión Flask: mismos códigos, mismos cuerpos de error.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::analysis::{interpretar_corrientes, potencia_total};
use crate::domain::{MeshParams, EXAMPLE_VALUES};
use crate::exporter::build_csv;
use crate::render::{build_pdf, build_png};
use crate::solver::{resolver, SINGULAR_MESSAGE};

pub const CAPACITY: usize = 50;
pub const SERVICE_NAME: &str = "simulador-circuitos-malla";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const REQUIRED: [&str; 9] = ["R1", "R2", "R3", "R4", "R5", "R6", "V1", "V2", "V3"];

/// Contrato de error uniforme: success, error.code, error.message, error.details.
pub enum ApiError {
    UnsupportedMediaType,
    MalformedJson,
    InvalidPayload(String),
    Calculation(String),
    NotFound(String),
}

impl ApiError {
    fn parts(&self) -> (StatusCode, &'static str, String, Option<String>) {
        match self {
            Self::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                "Content-Type debe ser application/json".to_string(),
                Some("Envia la solicitud con header Content-Type: application/json".to_string()),
            ),
            Self::MalformedJson => (
                StatusCode::BAD_REQUEST,
                "MALFORMED_JSON",
                "JSON malformado".to_string(),
                None,
            ),
            Self::InvalidPayload(detail) => (
                StatusCode::BAD_REQUEST,
                "INVALID_PAYLOAD",
                "Datos de entrada inválidos".to_string(),
                Some(detail.clone()),
            ),
            Self::Calculation(detail) => (
                StatusCode::BAD_REQUEST,
                "CALCULATION_ERROR",
                "Error de cálculo".to_string(),
                Some(detail.clone()),
            ),
            Self::NotFound(detail) => (StatusCode::NOT_FOUND, "NOT_FOUND", detail.clone(), None),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = self.parts();
        let body = json!({
            "success": false,
            "error": { "code": code, "message": message, "details": details }
        });
        (status, Json(body)).into_response()
    }
}

#[derive(Clone, Serialize)]
pub struct Entry {
    pub id: String,
    pub timestamp: String,
    pub params: MeshParams,
    pub currents: [f64; 3],
    pub interpretations: [String; 3],
    pub power_total: f64,
}

struct Store {
    entries: VecDeque<Entry>,
    counter: AtomicU64,
}

#[derive(Clone)]
pub struct AppState {
    store: Arc<Mutex<Store>>,
    capacity: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(Store {
                entries: VecDeque::new(),
                counter: AtomicU64::new(0),
            })),
            capacity: CAPACITY,
        }
    }

    pub fn record(
        &self,
        params: MeshParams,
        currents: [f64; 3],
        interpretations: [String; 3],
        power_total: f64,
    ) -> String {
        let mut store = self.store.lock().expect("el historial no esta envenenado");
        let seq = store.counter.fetch_add(1, Ordering::Relaxed);
        let id = format!("{seq:012x}");
        store.entries.push_front(Entry {
            id: id.clone(),
            timestamp: "1970-01-01T00:00:00Z".to_string(),
            params,
            currents,
            interpretations,
            power_total,
        });
        while store.entries.len() > self.capacity {
            store.entries.pop_back();
        }
        id
    }

    fn remove(&self, id: &str) -> Option<Entry> {
        let mut store = self.store.lock().expect("el historial no esta envenenado");
        if let Some(pos) = store.entries.iter().position(|e| e.id == id) {
            return store.entries.remove(pos);
        }
        None
    }

    fn clear(&self) -> usize {
        let mut store = self.store.lock().expect("el historial no esta envenenado");
        let n = store.entries.len();
        store.entries.clear();
        n
    }

    fn snapshot(&self) -> Vec<Entry> {
        let store = self.store.lock().expect("el historial no esta envenenado");
        store.entries.iter().cloned().collect()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprueba el Content-Type y parsea el cuerpo con los mismos errores que la versión Python.
fn parse_json(headers: &HeaderMap, body: &Bytes) -> Result<Map<String, Value>, ApiError> {
    let is_json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false);
    if !is_json {
        return Err(ApiError::UnsupportedMediaType);
    }
    match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err(ApiError::MalformedJson),
        Err(_) => Err(ApiError::MalformedJson),
    }
}

/// Replica validate_api_payload: faltantes primero, después conversión numérica, después rangos.
fn params_from(map: &Map<String, Value>) -> Result<MeshParams, ApiError> {
    if map.is_empty() {
        return Err(ApiError::InvalidPayload(
            "No se recibieron datos JSON".to_string(),
        ));
    }
    let missing: Vec<&str> = REQUIRED
        .iter()
        .copied()
        .filter(|k| !map.contains_key(*k))
        .collect();
    if !missing.is_empty() {
        // Python imprime la lista con comillas simples: ['R2', 'R3']. El cliente ve ese texto.
        let lista = missing
            .iter()
            .map(|key| format!("'{key}'"))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ApiError::InvalidPayload(format!(
            "Parámetros faltantes: [{lista}]"
        )));
    }
    let mut values = [0.0_f64; 9];
    for (index, key) in REQUIRED.iter().enumerate() {
        match map.get(*key).and_then(Value::as_f64) {
            Some(value) => values[index] = value,
            None => {
                return Err(ApiError::InvalidPayload(
                    "Los parámetros deben ser numéricos".to_string(),
                ));
            }
        }
    }
    let params = MeshParams {
        r1: values[0],
        r2: values[1],
        r3: values[2],
        r4: values[3],
        r5: values[4],
        r6: values[5],
        v1: values[6],
        v2: values[7],
        v3: values[8],
    };
    params
        .validate()
        .map_err(|e| ApiError::InvalidPayload(e.to_string()))?;
    Ok(params)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/version", get(version))
        .route("/api/example", get(example))
        .route("/api/calculate", post(calculate))
        .route("/api/history", get(history).delete(clear_history))
        .route("/api/history/{id}", delete(remove_entry))
        .route("/api/export/csv", post(export_csv))
        .route("/api/export/pdf", post(export_pdf))
        .route("/circuito.png", get(circuito_png))
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "service": SERVICE_NAME, "status": "ok" }))
}

async fn version() -> Json<Value> {
    Json(json!({ "service": SERVICE_NAME, "version": VERSION }))
}

async fn example() -> Json<Value> {
    Json(json!(EXAMPLE_VALUES))
}

async fn calculate(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let map = parse_json(&headers, &body)?;
    let params = params_from(&map)?;
    let currents = resolver(params.matrix(), params.vector())
        .ok_or_else(|| ApiError::Calculation(SINGULAR_MESSAGE.to_string()))?;
    let interpretations = interpretar_corrientes(currents);
    let power = potencia_total(currents, &params);
    let id = state.record(params, currents, interpretations.clone(), power);

    Ok(Json(json!({
        "success": true,
        "id": id,
        "currents": { "I1": currents[0], "I2": currents[1], "I3": currents[2] },
        "matrix_A": params.matrix(),
        "vector_B": params.vector(),
        "interpretations": { "I1": interpretations[0], "I2": interpretations[1], "I3": interpretations[2] },
        "power_total": power,
    })))
}

/// Acepta el id de una simulación guardada o los parámetros completos, igual que Python.
fn resolve_export(state: &AppState, map: &Map<String, Value>) -> Result<Entry, ApiError> {
    if let Some(Value::String(id)) = map.get("id") {
        return state
            .snapshot()
            .into_iter()
            .find(|e| &e.id == id)
            .ok_or_else(|| ApiError::NotFound("Simulación no encontrada".to_string()));
    }
    let params = params_from(map)?;
    let currents = resolver(params.matrix(), params.vector())
        .ok_or_else(|| ApiError::Calculation(SINGULAR_MESSAGE.to_string()))?;
    Ok(Entry {
        id: "ad-hoc".to_string(),
        timestamp: "1970-01-01T00:00:00Z".to_string(),
        params,
        currents,
        interpretations: interpretar_corrientes(currents),
        power_total: potencia_total(currents, &params),
    })
}

async fn export_csv(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    let map = parse_json(&headers, &body)?;
    let entry = resolve_export(&state, &map)?;
    let csv = build_csv(&entry);
    let filename = format!("simulacion_{}.csv", entry.id);
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        csv,
    )
        .into_response())
}

async fn export_pdf(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    let map = parse_json(&headers, &body)?;
    let entry = resolve_export(&state, &map)?;
    let pdf = build_pdf(&entry).map_err(ApiError::Calculation)?;
    let filename = format!("simulacion_{}.pdf", entry.id);
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        pdf,
    )
        .into_response())
}

/// Imagen del circuito. Acepta los mismos parámetros de consulta que la versión Python.
async fn circuito_png(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let mut mesh = EXAMPLE_VALUES;
    let leer = |clave: &str, actual: f64| -> Result<f64, ApiError> {
        match params.get(clave) {
            Some(valor) if !valor.trim().is_empty() => valor
                .replace(',', ".")
                .parse::<f64>()
                .map_err(|_| ApiError::Calculation(format!("{clave}: Debe ser un número válido"))),
            _ => Ok(actual),
        }
    };
    mesh.r1 = leer("R1", mesh.r1)?;
    mesh.r2 = leer("R2", mesh.r2)?;
    mesh.r3 = leer("R3", mesh.r3)?;
    mesh.r4 = leer("R4", mesh.r4)?;
    mesh.r5 = leer("R5", mesh.r5)?;
    mesh.r6 = leer("R6", mesh.r6)?;
    mesh.v1 = leer("V1", mesh.v1)?;
    mesh.v2 = leer("V2", mesh.v2)?;
    mesh.v3 = leer("V3", mesh.v3)?;
    mesh.validate()
        .map_err(|e| ApiError::Calculation(e.to_string()))?;

    let resistencias = params
        .get("resistance_count")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(6)
        .clamp(1, 6);
    let voltajes = params
        .get("voltage_count")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
        .clamp(1, 3);

    let png = build_png(&mesh, resistencias, voltajes).map_err(ApiError::Calculation)?;
    Ok(([(header::CONTENT_TYPE, "image/png".to_string())], png).into_response())
}

async fn history(State(state): State<AppState>) -> Json<Value> {
    let items = state.snapshot();
    Json(json!({
        "success": true,
        "count": items.len(),
        "capacity": CAPACITY,
        "items": items,
    }))
}

async fn clear_history(State(state): State<AppState>) -> Json<Value> {
    let removed = state.clear();
    Json(json!({ "success": true, "removed": removed }))
}

async fn remove_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    state
        .remove(&id)
        .map(|entry| Json(json!({ "success": true, "item": entry })))
        .ok_or_else(|| ApiError::NotFound("Simulación no encontrada".to_string()))
}
