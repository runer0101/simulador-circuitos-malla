//! Defensas del servidor completo: cabeceras, tamaño de petición y límite por cliente.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use simulador_circuitos_malla::app;
use simulador_circuitos_malla::seguridad::Limitador;
use tower::ServiceExt;

const EJEMPLO: &str = r#"{"R1":2,"R2":4,"R3":3,"R4":6,"R5":5,"R6":2,"V1":12,"V2":0,"V3":0}"#;

/// La raíz es el repositorio: las pruebas corren desde rust/ y las plantillas están un nivel arriba.
fn servidor() -> Router {
    app::construir("..")
}

async fn pedir_una(router: Router, request: Request<Body>) -> axum::response::Response {
    router.oneshot(request).await.expect("respuesta")
}

#[tokio::test]
async fn las_cabeceras_de_seguridad_estan_en_todas_las_respuestas() {
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .expect("peticion");
    let respuesta = pedir_una(servidor(), request).await;

    assert_eq!(respuesta.status(), StatusCode::OK);
    let h = respuesta.headers();
    let csp = h[header::CONTENT_SECURITY_POLICY].to_str().expect("csp");
    assert!(csp.contains("object-src 'none'"));
    assert!(csp.contains("frame-ancestors 'none'"));
    assert!(csp.contains("fonts.googleapis.com"));
    assert_eq!(h[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(h[header::X_FRAME_OPTIONS], "DENY");
    assert_eq!(h[header::REFERRER_POLICY], "same-origin");
}

#[tokio::test]
async fn una_peticion_demasiado_grande_se_rechaza() {
    let grande = vec![b'x'; 70 * 1024];
    let request = Request::builder()
        .method("POST")
        .uri("/api/calculate")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(grande))
        .expect("peticion");

    let respuesta = pedir_una(servidor(), request).await;

    assert_eq!(respuesta.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn el_limite_corta_tras_sesenta_peticiones_y_dice_cuando_volver() {
    // Techo explícito: la prueba no debe depender de RATE_LIMIT_REQUESTS del entorno.
    let router = app::construir_con("..", Limitador::nuevo(60, 60));
    let mut ultima = StatusCode::OK;

    for _ in 0..62 {
        let request = Request::builder()
            .method("POST")
            .uri("/api/calculate")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(EJEMPLO))
            .expect("peticion");
        let respuesta = router.clone().oneshot(request).await.expect("respuesta");
        ultima = respuesta.status();
        if ultima == StatusCode::TOO_MANY_REQUESTS {
            let cabecera = respuesta
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            assert!(
                cabecera.is_some_and(|s| s >= 1),
                "Retry-After ausente o inválido"
            );
            return;
        }
    }

    panic!("el límite no cortó: última respuesta {ultima}");
}

#[tokio::test]
async fn el_limite_configurado_corta_donde_se_le_pide() {
    // El techo configurado debe ser el que se aplica, no una constante fija.
    let router = app::construir_con("..", Limitador::nuevo(2, 60));

    for _ in 0..2 {
        let peticion = Request::builder()
            .method("POST")
            .uri("/api/calculate")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(EJEMPLO))
            .expect("peticion");
        let respuesta = router.clone().oneshot(peticion).await.expect("respuesta");
        assert_eq!(respuesta.status(), StatusCode::OK);
    }

    let tercera = Request::builder()
        .method("POST")
        .uri("/api/calculate")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(EJEMPLO))
        .expect("peticion");
    let respuesta = router.oneshot(tercera).await.expect("respuesta");
    assert_eq!(respuesta.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn un_limite_en_cero_desactiva_el_control() {
    // `RATE_LIMIT_REQUESTS=0` desactiva el límite, como documenta el README.
    let router = app::construir_con("..", Limitador::nuevo(0, 60));

    for _ in 0..70 {
        let peticion = Request::builder()
            .method("POST")
            .uri("/api/calculate")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(EJEMPLO))
            .expect("peticion");
        let respuesta = router.clone().oneshot(peticion).await.expect("respuesta");
        assert_eq!(respuesta.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn las_lecturas_no_consumen_el_limite() {
    let router = servidor();
    for _ in 0..80 {
        let request = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .expect("peticion");
        let respuesta = router.clone().oneshot(request).await.expect("respuesta");
        assert_eq!(respuesta.status(), StatusCode::OK);
    }
}
