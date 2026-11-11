use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use simulador_circuitos_malla::api::{router as api_router, AppState};
use simulador_circuitos_malla::web::{calcular_formulario, entorno, portada, WebState};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raiz = std::env::var("APP_ROOT").unwrap_or_else(|_| ".".to_string());
    let env = Arc::new(entorno(&raiz));
    let api = AppState::new();

    let web = Router::new()
        .route("/", get(portada).post(calcular_formulario))
        .with_state(WebState {
            env: env.clone(),
            api: api.clone(),
        });

    let app = api_router(api)
        .merge(web)
        .nest_service("/static", ServeDir::new(format!("{raiz}/static")));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("simulador-circuitos-malla escuchando en http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
