//! Armado del servidor completo: API, interfaz, estáticos y defensas.
//!
//! Vive en la librería, no en el binario, para que las pruebas puedan ejercitarlo entero.

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::get;
use axum::Router;
use tower_http::services::ServeDir;

use crate::api::{router as api_router, AppState};
use crate::seguridad::{self, Limitador, MAX_CONTENT_LENGTH};
use crate::web::{calcular_formulario, entorno, portada, WebState};

/// Construye la aplicación. `raiz` es el directorio que contiene `templates/` y `static/`.
pub fn construir(raiz: &str) -> Router {
    let env = Arc::new(entorno(raiz));
    let api = AppState::new();

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
            Limitador::new(),
            seguridad::limite,
        ))
        .layer(axum::middleware::from_fn(seguridad::cabeceras))
}
