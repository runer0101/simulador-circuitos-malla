//! Armado del servidor completo: API, interfaz, estáticos y defensas.
//!
//! Vive en la librería, no en el binario, para que las pruebas puedan ejercitarlo entero.

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
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
}
