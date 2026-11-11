//! Armado del servidor completo: API, interfaz, estáticos y defensas.
//!
//! Vive en la librería, no en el binario, para que las pruebas puedan ejercitarlo entero.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use tower_http::services::ServeDir;

use crate::api::{router as api_router, AppState};
use crate::seguridad::{self, Limitador, Metricas, MAX_CONTENT_LENGTH};
use crate::web::{calcular_formulario, entorno, portada, WebState};

/// Construye la aplicación con la configuración del entorno. `raiz` es el directorio que
/// contiene `templates/` y `static/`.
pub fn construir(raiz: &str) -> Router {
    construir_con(raiz, Limitador::desde_entorno())
}

/// Igual que [`construir`] pero con un limitador explícito, para pruebas y despliegues que fijen
/// el techo por código.
pub fn construir_con(raiz: &str, limitador: Limitador) -> Router {
    let env = Arc::new(entorno(raiz));
    let metricas = Metricas::new();
    let api = AppState::con_metricas(metricas.clone());

    let web = Router::new()
        .route("/", get(portada).post(calcular_formulario))
        .with_state(WebState {
            env,
            api: api.clone(),
        });

    api_router(api)
        .merge(web)
        .nest_service("/static", ServeDir::new(format!("{raiz}/static")))
        .layer(DefaultBodyLimit::max(MAX_CONTENT_LENGTH))
        .layer(axum::middleware::from_fn_with_state(
            limitador,
            seguridad::limite,
        ))
        .layer(axum::middleware::from_fn(seguridad::cabeceras))
        // El más externo: cuenta también lo que el limitador rechaza.
        .layer(axum::middleware::from_fn_with_state(
            metricas,
            seguridad::medir,
        ))
        .fallback(no_encontrado)
}

/// Una direccion que no existe responde en el formato de su familia: la API en JSON, el sitio en
/// HTML. Sin esto, quien escribia mal una direccion veia una pagina en blanco.
async fn no_encontrado(peticion: axum::http::Request<Body>) -> Response {
    if peticion.uri().path().starts_with("/api/") {
        let cuerpo = serde_json::json!({
            "success": false,
            "error": {
                "code": "NOT_FOUND",
                "message": "Dirección no encontrada",
                "details": null
            }
        })
        .to_string();
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            cuerpo,
        )
            .into_response();
    }

    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        PAGINA_NO_ENCONTRADA,
    )
        .into_response()
}

const PAGINA_NO_ENCONTRADA: &str = r#"<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Pagina no encontrada</title>
    <style>
        body {
            margin: 0;
            min-height: 100vh;
            display: grid;
            place-items: center;
            font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
            background: #f5f2ea;
            color: #2b2b28;
            padding: 1.5rem;
        }
        main { max-width: 32rem; }
        h1 { font-size: 1.6rem; margin: 0 0 0.75rem; }
        p { line-height: 1.6; margin: 0 0 1.25rem; color: #55524a; }
        a {
            display: inline-block;
            padding: 0.65rem 1.2rem;
            border-radius: 999px;
            background: #1f6f5c;
            color: #fff;
            text-decoration: none;
            font-weight: 600;
        }
        a:focus-visible { outline: 3px solid #1f6f5c; outline-offset: 3px; }
    </style>
</head>
<body>
    <main>
        <h1>Esta dirección no existe</h1>
        <p>El enlace puede estar mal escrito o el ejercicio ya no está disponible.</p>
        <a href="/">Volver a la calculadora</a>
    </main>
</body>
</html>
"#;
