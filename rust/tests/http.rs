//! Contrato HTTP: mismos códigos y mismos cuerpos de error que la versión Flask.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use serde_json::Value;
use simulador_circuitos_malla::api::{router, AppState};
use tower::ServiceExt;

const EJEMPLO: &str = r#"{"R1":2,"R2":4,"R3":3,"R4":6,"R5":5,"R6":2,"V1":12,"V2":0,"V3":0}"#;

fn app() -> Router {
    router(AppState::new())
}

async fn send(router: Router, method: &str, uri: &str, body: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let request = builder
        .body(Body::from(body.unwrap_or("").to_string()))
        .expect("peticion valida");
    let response = router.oneshot(request).await.expect("respuesta");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("cuerpo legible");
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn salud_responde_service_y_status() {
    let (status, body) = send(app(), "GET", "/api/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["service"], "simulador-circuitos-malla");
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn ejemplo_devuelve_los_nueve_parametros() {
    let (status, body) = send(app(), "GET", "/api/example", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["R1"], 2.0);
    assert_eq!(body["V1"], 12.0);
}

#[tokio::test]
async fn calculate_devuelve_las_mismas_corrientes_que_python() {
    let (status, body) = send(app(), "POST", "/api/calculate", Some(EJEMPLO)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    let i1 = body["currents"]["I1"].as_f64().expect("numero");
    assert!((i1 - 2.112_676_056_338_028_5).abs() < 1e-9);
    assert!(body["id"].is_string());
    assert_eq!(
        body["interpretations"]["I1"],
        "Sala/Comedor: 2.11A (horario) - Carga normal"
    );
}

#[tokio::test]
async fn sin_content_type_json_responde_415() {
    let request = Request::builder()
        .method("POST")
        .uri("/api/calculate")
        .body(Body::from(EJEMPLO))
        .expect("peticion");
    let response = app().oneshot(request).await.expect("respuesta");

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("cuerpo");
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(body["error"]["code"], "UNSUPPORTED_MEDIA_TYPE");
}

#[tokio::test]
async fn json_malformado_responde_400_malformed_json() {
    let (status, body) = send(app(), "POST", "/api/calculate", Some("{no es json")).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "MALFORMED_JSON");
}

#[tokio::test]
async fn parametros_faltantes_nombran_lo_que_falta() {
    let (status, body) = send(app(), "POST", "/api/calculate", Some(r#"{"R1":2}"#)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "INVALID_PAYLOAD");
    // El texto debe ser idéntico al de Python, comillas simples incluidas.
    assert_eq!(
        body["error"]["details"],
        "Parámetros faltantes: ['R2', 'R3', 'R4', 'R5', 'R6', 'V1', 'V2', 'V3']"
    );
}

#[tokio::test]
async fn un_valor_fuera_de_rango_es_dato_invalido_y_nombra_el_mensaje_python() {
    let (status, body) = send(
        app(),
        "POST",
        "/api/calculate",
        Some(r#"{"R1":0,"R2":4,"R3":3,"R4":6,"R5":5,"R6":2,"V1":12,"V2":0,"V3":0}"#),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "INVALID_PAYLOAD");
    assert_eq!(body["error"]["details"], "R1: Debe ser un valor positivo");
}

#[tokio::test]
async fn el_historial_guarda_la_simulacion_y_se_puede_limpiar() {
    let router = app();

    let (_, vacio) = send(router.clone(), "GET", "/api/history", None).await;
    assert_eq!(vacio["count"], 0);

    send(router.clone(), "POST", "/api/calculate", Some(EJEMPLO)).await;
    let (_, con_uno) = send(router.clone(), "GET", "/api/history", None).await;
    assert_eq!(con_uno["count"], 1);
    assert_eq!(con_uno["capacity"], 50);

    let (_, limpiado) = send(router, "DELETE", "/api/history", None).await;
    assert_eq!(limpiado["removed"], 1);
}

#[tokio::test]
async fn borrar_una_simulacion_inexistente_responde_404() {
    let (status, body) = send(app(), "DELETE", "/api/history/no-existe", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn export_csv_devuelve_un_archivo_descargable() {
    let request = Request::builder()
        .method("POST")
        .uri("/api/export/csv")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(EJEMPLO))
        .expect("peticion");
    let response = app().oneshot(request).await.expect("respuesta");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "text/csv");
    let disposition = response.headers()[header::CONTENT_DISPOSITION]
        .to_str()
        .expect("cabecera");
    assert!(disposition.contains("simulacion_ad-hoc.csv"));
}

#[tokio::test]
async fn export_pdf_devuelve_un_pdf() {
    let request = Request::builder()
        .method("POST")
        .uri("/api/export/pdf")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(EJEMPLO))
        .expect("peticion");
    let response = app().oneshot(request).await.expect("respuesta");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/pdf");
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("cuerpo");
    assert!(bytes.starts_with(b"%PDF"));
}

#[tokio::test]
async fn circuito_png_devuelve_una_imagen() {
    let request = Request::builder()
        .method("GET")
        .uri("/circuito.png?R1=2&R2=4&R3=3&R4=6&R5=5&R6=2&V1=12")
        .body(Body::empty())
        .expect("peticion");
    let response = app().oneshot(request).await.expect("respuesta");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "image/png");
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("cuerpo");
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
}

#[tokio::test]
async fn export_de_un_id_inexistente_responde_404() {
    let (status, body) = send(app(), "POST", "/api/export/csv", Some(r#"{"id":"nope"}"#)).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}
