//! Cabeceras de seguridad y límite de peticiones, equivalentes a los de la versión Python.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub const MAX_CONTENT_LENGTH: usize = 64 * 1024;
pub const RATE_LIMIT_REQUESTS: usize = 60;
pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 60;

const CSP: &str = "default-src 'self'; \
script-src 'self' https://cdn.jsdelivr.net; \
style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
img-src 'self' data:; \
font-src 'self' data: https://cdn.jsdelivr.net https://fonts.gstatic.com; \
connect-src 'self'; \
object-src 'none'; \
base-uri 'self'; \
form-action 'self'; \
frame-ancestors 'none'";

/// Añade las mismas cabeceras que la implementación Python.
pub async fn cabeceras(peticion: Request, siguiente: Next) -> Response {
    let mut respuesta = siguiente.run(peticion).await;
    let h = respuesta.headers_mut();
    if let Ok(v) = HeaderValue::from_str(CSP) {
        h.insert(header::CONTENT_SECURITY_POLICY, v);
    }
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    h.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );

    // El HTML no se cachea: si el navegador guarda una version anterior, el usuario no ve los
    // cambios aunque esten publicados.
    let es_html = h
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("text/html"))
        .unwrap_or(false);
    if es_html {
        h.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, must-revalidate"),
        );
    }
    respuesta
}

/// Ventana deslizante por cliente.
#[derive(Clone)]
pub struct Limitador {
    inner: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl Limitador {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn permite(&self, clave: &str, limite: usize, ventana: u64) -> Option<u64> {
        let mut mapa = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ahora = Instant::now();
        let marcas = mapa.entry(clave.to_string()).or_default();
        while let Some(primera) = marcas.front() {
            if ahora.duration_since(*primera).as_secs() >= ventana {
                marcas.pop_front();
            } else {
                break;
            }
        }
        if marcas.len() >= limite {
            let espera = ventana - ahora.duration_since(*marcas.front()?).as_secs();
            return Some(espera.max(1));
        }
        marcas.push_back(ahora);
        if mapa.len() > 4096 {
            mapa.retain(|_, v| {
                v.back()
                    .is_some_and(|t| ahora.duration_since(*t).as_secs() < ventana)
            });
        }
        None
    }
}

impl Default for Limitador {
    fn default() -> Self {
        Self::new()
    }
}

/// Cliente real: detrás del proxy de Render viene en X-Forwarded-For.
fn cliente(cabeceras: &axum::http::HeaderMap) -> String {
    cabeceras
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        // La ultima entrada la anade el proxy; la primera la escribe el cliente.
        .and_then(|v| v.split(',').next_back())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "desconocido".to_string())
}

fn es_cara(metodo: &axum::http::Method, ruta: &str) -> bool {
    ruta == "/circuito.png"
        || (metodo == axum::http::Method::POST && (ruta.starts_with("/api/") || ruta == "/"))
}

/// Aplica el límite a lo que consume CPU: cálculo, exportación y render.
pub async fn limite(
    State(limitador): State<Limitador>,
    peticion: Request,
    siguiente: Next,
) -> Response {
    let metodo = peticion.method().clone();
    let ruta = peticion.uri().path().to_string();
    if !es_cara(&metodo, &ruta) {
        return siguiente.run(peticion).await;
    }
    let clave = cliente(peticion.headers());
    if let Some(espera) = limitador.permite(&clave, RATE_LIMIT_REQUESTS, RATE_LIMIT_WINDOW_SECONDS)
    {
        // El formulario HTML necesita una pagina; la API, su contrato JSON.
        let mut respuesta = if ruta == "/" {
            let html = format!(
                "<!doctype html><html lang=\"es\"><head><meta charset=\"utf-8\">\
                 <title>Demasiadas peticiones</title>\
                 <style>body{{font-family:system-ui,sans-serif;max-width:34rem;margin:4rem auto;\
                 padding:0 1.2rem;line-height:1.6}}h1{{font-size:1.3rem}}\
                 a{{color:#0f766e}}</style></head><body>\
                 <h1>Demasiadas peticiones seguidas</h1>\
                 <p>Ha enviado más cálculos de los permitidos en poco tiempo.\
                 Espere {espera} segundos y vuelva a intentarlo.</p>\
                 <p><a href=\"/\">Volver al simulador</a></p></body></html>"
            );
            (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                html,
            )
                .into_response()
        } else {
            let cuerpo = json!({
                "success": false,
                "error": {
                    "code": "RATE_LIMITED",
                    "message": "Demasiadas peticiones seguidas. Espere antes de volver a intentarlo.",
                    "details": format!("límite de {RATE_LIMIT_REQUESTS} peticiones cada {RATE_LIMIT_WINDOW_SECONDS} s"),
                }
            });
            (StatusCode::TOO_MANY_REQUESTS, Json(cuerpo)).into_response()
        };
        respuesta.headers_mut().insert(
            header::RETRY_AFTER,
            HeaderValue::from_str(&espera.to_string()).unwrap_or(HeaderValue::from_static("1")),
        );
        return respuesta;
    }
    siguiente.run(peticion).await
}
