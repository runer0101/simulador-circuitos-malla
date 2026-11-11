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

// --- La página renderizada: lo que antes se comprobaba con pytest ---

/// Corrientes muy por encima de la banda crítica (50 A).
const CORRIENTE_ALTA: &str = "R1=0,01&R2=0,01&R3=0,01&R4=0,01&R5=0,01&R6=0,01&V1=500&V2=500&V3=500";

#[tokio::test]
async fn la_portada_ofrece_el_cambio_de_tema_y_el_punto_de_anuncios() {
    let (estado, _, html) = pedir(servidor(), "GET", "/", None, "").await;

    assert_eq!(estado, StatusCode::OK);
    for pieza in [
        r#"id="theme-toggle""#,
        "Cambiar tema",
        r#"aria-live="polite""#,
    ] {
        assert!(html.contains(pieza), "falta {pieza} en la portada");
    }
}

#[tokio::test]
async fn cada_corriente_dibujada_declara_su_severidad() {
    let (estado, _, html) = pedir(servidor(), "POST", "/", Some(FORMULARIO), NUEVE_CAMPOS).await;

    assert_eq!(estado, StatusCode::OK);
    assert!(
        html.contains("data-severity="),
        "las corrientes no declaran severidad"
    );
    // Con el ejemplo las corrientes son pequeñas: nunca la banda crítica.
    // Cada corriente dibujada declara una de las cuatro bandas, y ninguna otra cosa.
    let bandas = ["baja", "normal", "alta", "critica"];
    let declaradas: Vec<&str> = html
        .match_indices(r#"data-severity=""#)
        .map(|(i, m)| {
            let resto = &html[i + m.len()..];
            &resto[..resto.find('"').expect("comilla de cierre")]
        })
        .collect();
    assert!(
        !declaradas.is_empty(),
        "ninguna corriente declara severidad"
    );
    for banda in &declaradas {
        assert!(bandas.contains(banda), "banda desconocida: {banda}");
    }
}

#[tokio::test]
async fn una_corriente_alta_se_marca_como_critica_y_avisa() {
    let (estado, _, html) = pedir(servidor(), "POST", "/", Some(FORMULARIO), CORRIENTE_ALTA).await;

    assert_eq!(estado, StatusCode::OK);
    assert!(
        html.contains(r#"data-severity="critica""#),
        "una corriente alta no se marca como crítica"
    );
    assert!(
        html.contains("aviso-critico") && html.contains(r#"role="alert""#),
        "una corriente crítica no muestra el aviso"
    );
}

#[tokio::test]
async fn el_error_del_formulario_se_anuncia_a_lectores_de_pantalla() {
    let invalido = "R1=abc&R2=0,7&R3=0,6&R4=20&R5=15&R6=25&V1=120&V2=220&V3=120";
    let (estado, _, html) = pedir(servidor(), "POST", "/", Some(FORMULARIO), invalido).await;

    assert_eq!(estado, StatusCode::OK);
    assert!(html.contains(r#"role="alert""#), "el error no se anuncia");
    assert!(
        html.contains("Debe ser un número válido"),
        "mensaje ausente: {html}"
    );
}

#[tokio::test]
async fn una_direccion_del_sitio_que_no_existe_responde_en_html() {
    let (estado, cabeceras, _) = pedir(servidor(), "GET", "/no-existe", None, "").await;

    assert_eq!(estado, StatusCode::NOT_FOUND);
    let tipo = cabeceras
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(tipo.starts_with("text/html"), "tipo inesperado: {tipo}");
}

#[tokio::test]
async fn una_ruta_de_api_que_no_existe_responde_en_json() {
    let (estado, cabeceras, cuerpo) = pedir(servidor(), "GET", "/api/no-existe", None, "").await;

    assert_eq!(estado, StatusCode::NOT_FOUND);
    let tipo = cabeceras
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        tipo.starts_with("application/json"),
        "tipo inesperado: {tipo}"
    );
    assert!(
        serde_json::from_str::<serde_json::Value>(&cuerpo).is_ok(),
        "el cuerpo no es JSON: {cuerpo}"
    );
}
