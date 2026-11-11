// Generado a partir de la implementacion Python, que es el oraculo de la migracion.
use simulador_circuitos_malla::{interpretar_corrientes, potencia_total, resolver, MeshParams};

#[test]
fn paridad_numerica_ejemplo() {
    let p = MeshParams {
        r1: 2.0,
        r2: 4.0,
        r3: 3.0,
        r4: 6.0,
        r5: 5.0,
        r6: 2.0,
        v1: 12.0,
        v2: 0.0,
        v3: 0.0,
    };
    p.validate().expect("parametros validos");
    let x = resolver(p.matrix(), p.vector()).expect("sistema con solucion");
    let esperado: [f64; 3] = [2.1126760563380285, 1.1830985915492962, 1.0140845070422537];
    for (obtenido, esperado) in x.into_iter().zip(esperado) {
        let tolerancia = esperado.abs().max(1.0) * 1e-9;
        assert!(
            (obtenido - esperado).abs() <= tolerancia,
            "{obtenido} != {esperado}"
        );
    }
    assert!((potencia_total(x, &p) - 25.352112676056347).abs() < 1e-6);
    let textos = interpretar_corrientes(x);
    assert_eq!(textos[0], "Sala/Comedor: 2.11A (horario) - Carga normal");
    assert_eq!(
        textos[1],
        "Cocina/Lavandería: 1.18A (horario) - Carga normal"
    );
    assert_eq!(textos[2], "Dormitorios: 1.01A (horario) - Carga normal");
}

#[test]
fn paridad_numerica_defecto() {
    let p = MeshParams {
        r1: 0.5,
        r2: 0.7,
        r3: 0.6,
        r4: 20.0,
        r5: 15.0,
        r6: 25.0,
        v1: 120.0,
        v2: 220.0,
        v3: 120.0,
    };
    p.validate().expect("parametros validos");
    let x = resolver(p.matrix(), p.vector()).expect("sistema con solucion");
    let esperado: [f64; 3] = [255.33579479387248, 256.29733949161783, 254.87327493155368];
    for (obtenido, esperado) in x.into_iter().zip(esperado) {
        let tolerancia = esperado.abs().max(1.0) * 1e-9;
        assert!(
            (obtenido - esperado).abs() <= tolerancia,
            "{obtenido} != {esperado}"
        );
    }
    assert!((potencia_total(x, &p) - 117610.50305520729).abs() < 1e-6);
    let textos = interpretar_corrientes(x);
    assert_eq!(
        textos[0],
        "Sala/Comedor: 255.3A (horario) - [!] CARGA CRÍTICA"
    );
    assert_eq!(
        textos[1],
        "Cocina/Lavandería: 256.3A (horario) - [!] CARGA CRÍTICA"
    );
    assert_eq!(
        textos[2],
        "Dormitorios: 254.9A (horario) - [!] CARGA CRÍTICA"
    );
}

#[test]
fn paridad_numerica_limites_min() {
    let p = MeshParams {
        r1: 0.01,
        r2: 0.01,
        r3: 0.01,
        r4: 0.01,
        r5: 0.01,
        r6: 0.01,
        v1: 500.0,
        v2: 500.0,
        v3: 500.0,
    };
    p.validate().expect("parametros validos");
    let x = resolver(p.matrix(), p.vector()).expect("sistema con solucion");
    let esperado: [f64; 3] = [50000.00000000001, 50000.00000000001, 50000.00000000001];
    for (obtenido, esperado) in x.into_iter().zip(esperado) {
        let tolerancia = esperado.abs().max(1.0) * 1e-9;
        assert!(
            (obtenido - esperado).abs() <= tolerancia,
            "{obtenido} != {esperado}"
        );
    }
    assert!((potencia_total(x, &p) - 75000000.00000003).abs() < 1e-6);
    let textos = interpretar_corrientes(x);
    assert_eq!(
        textos[0],
        "Sala/Comedor: 50000.0A (horario) - [!] CARGA CRÍTICA"
    );
    assert_eq!(
        textos[1],
        "Cocina/Lavandería: 50000.0A (horario) - [!] CARGA CRÍTICA"
    );
    assert_eq!(
        textos[2],
        "Dormitorios: 50000.0A (horario) - [!] CARGA CRÍTICA"
    );
}

#[test]
fn paridad_numerica_limites_max() {
    let p = MeshParams {
        r1: 1000.0,
        r2: 1000.0,
        r3: 1000.0,
        r4: 1000.0,
        r5: 1000.0,
        r6: 1000.0,
        v1: 500.0,
        v2: 0.0,
        v3: 0.0,
    };
    p.validate().expect("parametros validos");
    let x = resolver(p.matrix(), p.vector()).expect("sistema con solucion");
    let esperado: [f64; 3] = [0.25, 0.125, 0.125];
    for (obtenido, esperado) in x.into_iter().zip(esperado) {
        let tolerancia = esperado.abs().max(1.0) * 1e-9;
        assert!(
            (obtenido - esperado).abs() <= tolerancia,
            "{obtenido} != {esperado}"
        );
    }
    assert!((potencia_total(x, &p) - 125.0).abs() < 1e-6);
    let textos = interpretar_corrientes(x);
    assert_eq!(textos[0], "Sala/Comedor: 0.250A (horario) - Carga baja");
    assert_eq!(
        textos[1],
        "Cocina/Lavandería: 0.125A (horario) - Carga baja"
    );
    assert_eq!(textos[2], "Dormitorios: 0.125A (horario) - Carga baja");
}

#[test]
fn paridad_numerica_asimetrico() {
    let p = MeshParams {
        r1: 1.5,
        r2: 2.5,
        r3: 3.5,
        r4: 4.5,
        r5: 5.5,
        r6: 6.5,
        v1: 10.0,
        v2: 20.0,
        v3: 30.0,
    };
    p.validate().expect("parametros validos");
    let x = resolver(p.matrix(), p.vector()).expect("sistema con solucion");
    let esperado: [f64; 3] = [7.86931818181818, 7.982954545454544, 8.068181818181817];
    for (obtenido, esperado) in x.into_iter().zip(esperado) {
        let tolerancia = esperado.abs().max(1.0) * 1e-9;
        assert!(
            (obtenido - esperado).abs() <= tolerancia,
            "{obtenido} != {esperado}"
        );
    }
    assert!((potencia_total(x, &p) - 480.3977272727272).abs() < 1e-6);
    let textos = interpretar_corrientes(x);
    assert_eq!(textos[0], "Sala/Comedor: 7.87A (horario) - Carga normal");
    assert_eq!(
        textos[1],
        "Cocina/Lavandería: 7.98A (horario) - Carga normal"
    );
    assert_eq!(textos[2], "Dormitorios: 8.07A (horario) - Carga normal");
}

#[test]
fn los_rangos_producen_los_mismos_mensajes() {
    let base = MeshParams {
        r1: 1.0,
        r2: 1.0,
        r3: 1.0,
        r4: 1.0,
        r5: 1.0,
        r6: 1.0,
        ..MeshParams::default()
    };
    let casos = [
        (
            MeshParams { r1: 0.0, ..base },
            "R1: Debe ser un valor positivo",
        ),
        (
            MeshParams { r1: 2000.0, ..base },
            "R1: Resistencia debe estar entre 0.01Ω y 1000.0Ω",
        ),
        (
            MeshParams { v1: 900.0, ..base },
            "V1: Voltaje debe estar entre 0.0V y 500.0V",
        ),
        (
            MeshParams {
                r3: f64::NAN,
                ..base
            },
            "R3: Debe ser un número válido",
        ),
    ];
    for (p, esperado) in casos {
        assert_eq!(p.validate().unwrap_err().to_string(), esperado);
    }
}
