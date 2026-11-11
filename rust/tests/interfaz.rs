//! El formulario HTML y la imagen del circuito comparten los mismos criterios de parseo que la
//! API: nueve campos, coma decimal aceptada y el mismo mensaje cuando un valor no es numérico.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use simulador_circuitos_malla::app;
use tower::ServiceExt;

const FORMULARIO: &str = "application/x-www-form-urlencoded";
const NUEVE_CAMPOS: &str = "R1=0,5&R2=0,7&R3=0,6&R4=20&R5=15&R6=25&V1=120&V2=220&V3=120";

fn servidor() -> Router {
    app::construir("..")
}

async fn pedir(
    router: Router,
    metodo: &str,
    uri: &str,
    tipo: Option<&str>,
    cuerpo: &str,
) -> (StatusCode, axum::http::HeaderMap, String) {
    let mut builder = Request::builder().method(metodo).uri(uri);
    if let Some(tipo) = tipo {
        builder = builder.header(header::CONTENT_TYPE, tipo);
    }
    let peticion = builder
        .body(Body::from(cuerpo.to_string()))
        .expect("petición válida");
    let respuesta = router.oneshot(peticion).await.expect("respuesta");
    let estado = respuesta.status();
    let cabeceras = respuesta.headers().clone();
    let bytes = axum::body::to_bytes(respuesta.into_body(), usize::MAX)
        .await
        .expect("cuerpo legible");
    (
        estado,
        cabeceras,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

#[tokio::test]
async fn el_formulario_acepta_coma_decimal_y_registra_una_simulacion() {
    let router = servidor();
    let (estado, _, html) =
        pedir(router.clone(), "POST", "/", Some(FORMULARIO), NUEVE_CAMPOS).await;

    assert_eq!(estado, StatusCode::OK);
    assert!(html.contains("Sala/Comedor"), "sin resultados: {html}");
    let (_, _, historial) = pedir(router, "GET", "/api/history", None, "").await;
    let historial: serde_json::Value = serde_json::from_str(&historial).expect("json");
    assert_eq!(historial["count"], 1);
}

#[tokio::test]
async fn el_formulario_rechaza_un_numero_invalido_sin_registrarlo() {
    let router = servidor();
    let (estado, _, html) = pedir(
        router.clone(),
        "POST",
        "/",
        Some(FORMULARIO),
        "R1=abc&R2=0,7&R3=0,6&R4=20&R5=15&R6=25&V1=120&V2=220&V3=120",
    )
    .await;

    assert_eq!(estado, StatusCode::OK);
    assert!(
        html.contains("R1: Debe ser un número válido"),
        "mensaje ausente: {html}"
    );
    let (_, _, historial) = pedir(router, "GET", "/api/history", None, "").await;
    let historial: serde_json::Value = serde_json::from_str(&historial).expect("json");
    assert_eq!(historial["count"], 0);
}

#[tokio::test]
async fn el_png_del_circuito_acepta_coma_decimal() {
    let (estado, cabeceras, _) = pedir(
        servidor(),
        "GET",
        "/circuito.png?R1=0,5&R2=0,7&R3=0,6&R4=20&R5=15&R6=25&V1=120&V2=220&V3=120",
        None,
        "",
    )
    .await;

    assert_eq!(estado, StatusCode::OK);
    assert_eq!(cabeceras[header::CONTENT_TYPE], "image/png");
}
