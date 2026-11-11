//! Interfaz web: reutiliza la plantilla Jinja del proyecto mediante minijinja.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use minijinja::value::{Kwargs, Value};
use minijinja::{Environment, Error};

use crate::analysis::interpretar_corrientes;
use crate::analisis_estudiante::{corrientes_por_resistencia, potencia_por_resistencia, verificar_lvk};
use crate::api::AppState;
use crate::domain::{MeshParams, DEFAULT_VALUES};
use crate::solver::{resolver, SINGULAR_MESSAGE};

pub const ERROR_FORM_PARSE: &str =
    "Error al procesar los datos. Verifica el formato de los números.";
pub const ERROR_INVALID_NUMBER: &str = "Debe ser un número válido";

/// Entorno de plantillas con `url_for` resuelto al directorio de estáticos.
pub fn entorno(raiz: &str) -> Environment<'static> {
    let mut env = Environment::new();
    // La plantilla llama a url_for de dos maneras: con filename (estáticos) y sin argumentos
    // (la imagen del circuito). Se resuelven las dos.
    env.add_function(
        "url_for",
        |args: minijinja::value::Rest<Value>, kwargs: Kwargs| -> Result<String, Error> {
            let endpoint = args
                .0
                .first()
                .map(|valor| valor.to_string())
                .unwrap_or_default();
            if endpoint.contains("circuito_png") {
                return Ok("/circuito.png".to_string());
            }
            if endpoint == "static" {
                let filename: String = kwargs.get("filename")?;
                // Se acepta el parámetro de versión de la plantilla, aunque la URL no lo lleve.
                let _version = kwargs.get::<Option<String>>("v")?;
                return Ok(format!("/static/{filename}"));
            }
            Ok(format!("/{endpoint}"))
        },
    );
    // Jinja en Python aplica el operador % a las cadenas: "%.3f"|format(x). minijinja interpreta
    // format como str.format, así que se repone el comportamiento printf que usa la plantilla.
    env.add_filter(
        "format",
        |formato: String, args: minijinja::value::Rest<Value>| -> Result<String, Error> {
            let Some(valor) = args.0.first() else {
                return Ok(formato);
            };
            let Some(pos) = formato.find("%.") else {
                return Ok(formato.replace("%s", &valor.to_string()));
            };
            let resto = &formato[pos + 2..];
            let digitos: String = resto.chars().take_while(|c| c.is_ascii_digit()).collect();
            let precision: usize = digitos.parse().unwrap_or(6);
            let salto = digitos.len() + 1; // los dígitos más la letra del tipo
            let numero: f64 = valor.to_string().parse().unwrap_or(0.0);
            let formateado = format!("{numero:.precision$}");
            Ok(format!(
                "{}{}{}",
                &formato[..pos],
                formateado,
                &resto[salto..]
            ))
        },
    );

    // Los errores de plantilla deben verse en el desarrollo, no silenciarse.
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    let ruta = format!("{raiz}/templates/index.html");
    let fuente = std::fs::read_to_string(&ruta).unwrap_or_else(|e| {
        panic!("no se pudo leer la plantilla {ruta}: {e}");
    });
    env.add_template_owned("index.html", fuente)
        .expect("la plantilla debe compilar");
    env
}

fn valores_a_json(params: &MeshParams) -> serde_json::Value {
    serde_json::json!({
        "R1": params.r1, "R2": params.r2, "R3": params.r3,
        "R4": params.r4, "R5": params.r5, "R6": params.r6,
        "V1": params.v1, "V2": params.v2, "V3": params.v3,
    })
}

fn contexto_base(params: &MeshParams) -> serde_json::Value {
    serde_json::json!({
        "vals": valores_a_json(params),
        "default_vals": valores_a_json(&DEFAULT_VALUES),
        "error": Value::from(Option::<String>::None),
        "I1": Value::from(()),
        "I2": Value::from(()),
        "I3": Value::from(()),
        "A": Value::from(()),
        "B": Value::from(()),
        "interpretaciones": Value::from(()),
        "last_simulation_id": Value::from(()),
    })
}

fn render(env: &Environment<'static>, contexto: serde_json::Value) -> Response {
    let plantilla = env
        .get_template("index.html")
        .expect("plantilla registrada");
    match plantilla.render(contexto) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("error de plantilla: {e:#}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error interno del servidor",
            )
                .into_response()
        }
    }
}

/// GET /: formulario con los valores por defecto.
pub async fn portada(State(state): State<WebState>) -> Response {
    render(&state.env, contexto_base(&DEFAULT_VALUES))
}

/// Convierte el formulario con los mismos criterios que `parse_form_data` en Python.
fn parse_form(form: &HashMap<String, String>) -> Result<MeshParams, String> {
    let mut p = DEFAULT_VALUES;
    let leer = |clave: &str, actual: f64| -> Result<f64, String> {
        match form.get(clave).map(|v| v.trim()).filter(|v| !v.is_empty()) {
            Some(texto) => texto
                .replace(',', ".")
                .parse::<f64>()
                .map_err(|_| format!("{clave}: {ERROR_INVALID_NUMBER}")),
            None => Ok(actual),
        }
    };
    p.r1 = leer("R1", p.r1)?;
    p.r2 = leer("R2", p.r2)?;
    p.r3 = leer("R3", p.r3)?;
    p.r4 = leer("R4", p.r4)?;
    p.r5 = leer("R5", p.r5)?;
    p.r6 = leer("R6", p.r6)?;
    p.v1 = leer("V1", p.v1)?;
    p.v2 = leer("V2", p.v2)?;
    p.v3 = leer("V3", p.v3)?;
    p.validate().map_err(|e| e.to_string())?;
    Ok(p)
}

/// POST /: calcula y devuelve la página con los resultados, igual que la versión Python.
pub async fn calcular_formulario(
    State(state): State<WebState>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let env = state.env.clone();
    let params = match parse_form(&form) {
        Ok(p) => p,
        Err(mensaje) => {
            let mut ctx = contexto_base(&DEFAULT_VALUES);
            ctx["error"] = serde_json::Value::String(mensaje);
            return render(&env, ctx);
        }
    };

    let Some(corrientes) = resolver(params.matrix(), params.vector()) else {
        let mut ctx = contexto_base(&params);
        ctx["error"] = serde_json::Value::String(SINGULAR_MESSAGE.to_string());
        return render(&env, ctx);
    };

    let interpretaciones = interpretar_corrientes(corrientes);
    let potencia = crate::analysis::potencia_total(corrientes, &params);
    let id = state
        .api
        .record(params, corrientes, interpretaciones.clone(), potencia);

    let mut ctx = contexto_base(&params);
    ctx["I1"] = serde_json::json!(corrientes[0]);
    ctx["I2"] = serde_json::json!(corrientes[1]);
    ctx["I3"] = serde_json::json!(corrientes[2]);
    ctx["A"] = serde_json::json!(params.matrix());
    ctx["B"] = serde_json::json!(params.vector());
    ctx["interpretaciones"] = serde_json::json!({
        "I1": interpretaciones[0], "I2": interpretaciones[1], "I3": interpretaciones[2],
    });
    ctx["last_simulation_id"] = serde_json::Value::String(id);

    // Datos que piden los trabajos: corriente y potencia por resistencia, y la comprobación
    // de la ley de tensiones en cada malla.
    let ramas = corrientes_por_resistencia(corrientes, &params);
    let potencias = potencia_por_resistencia(corrientes, &params);
    let verificaciones = verificar_lvk(corrientes, &params);
    ctx["ramas"] = serde_json::json!({
        "R1": ramas[0], "R2": ramas[1], "R3": ramas[2],
        "R4": ramas[3], "R5": ramas[4], "R6": ramas[5],
    });
    ctx["potencias"] = serde_json::json!({
        "R1": potencias[0], "R2": potencias[1], "R3": potencias[2],
        "R4": potencias[3], "R5": potencias[4], "R6": potencias[5],
    });
    ctx["verificacion"] = serde_json::json!(verificaciones
        .iter()
        .map(|v| serde_json::json!({
            "malla": v.malla, "fuente": v.fuente, "caidas": v.caidas, "residuo": v.residuo,
        }))
        .collect::<Vec<_>>());
    ctx["sistema_a"] = serde_json::json!(params.matrix());
    ctx["sistema_b"] = serde_json::json!(params.vector());
    render(&env, ctx)
}

/// Estado compartido por la interfaz.
#[derive(Clone)]
pub struct WebState {
    pub env: Arc<Environment<'static>>,
    pub api: AppState,
}
