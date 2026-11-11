use std::net::SocketAddr;

use simulador_circuitos_malla::api::{router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("simulador-circuitos-malla escuchando en http://{addr}");
    axum::serve(listener, router(AppState::new())).await?;
    Ok(())
}
