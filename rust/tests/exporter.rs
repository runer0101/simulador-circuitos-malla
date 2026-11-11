//! El CSV debe ser idéntico al que produce el exportador Python.

use simulador_circuitos_malla::api::Entry;
use simulador_circuitos_malla::domain::MeshParams;
use simulador_circuitos_malla::exporter::build_csv;

fn entry_ejemplo() -> Entry {
    let params = MeshParams {
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
    Entry {
        id: "ad-hoc".to_string(),
        timestamp: "1970-01-01T00:00:00Z".to_string(),
        params,
        currents: [
            2.112_676_056_338_028_5,
            1.183_098_591_549_296_2,
            1.014_084_507_042_253_7,
        ]
        .into(),
        interpretations: [
            "Sala/Comedor: 2.11A (horario) - Carga normal".to_string(),
            "Cocina/Lavandería: 1.18A (horario) - Carga normal".to_string(),
            "Dormitorios: 1.01A (horario) - Carga normal".to_string(),
        ]
        .into(),
        power_total: 25.352_112_644_099_93,
    }
}

#[test]
fn el_csv_coincide_con_el_de_python() {
    let esperado = "simulador-circuitos-malla
id,ad-hoc
timestamp,1970-01-01T00:00:00Z

Malla,Corriente (A),Interpretación
I1,2.112676,Sala/Comedor: 2.11A (horario) - Carga normal
I2,1.183099,Cocina/Lavandería: 1.18A (horario) - Carga normal
I3,1.014085,Dormitorios: 1.01A (horario) - Carga normal
Potencia total (W),25.3521,
--- Parámetros ---,,
R1,2.0000,
R2,4.0000,
R3,3.0000,
R4,6.0000,
R5,5.0000,
R6,2.0000,
V1,12.0000,
V2,0.0000,
V3,0.0000,
"
    .replace('\n', "\r\n");

    let obtenido = String::from_utf8(build_csv(&entry_ejemplo())).expect("utf-8");

    assert_eq!(obtenido, esperado);
}

#[test]
fn los_campos_con_coma_o_comilla_se_escapan() {
    let mut entry = entry_ejemplo();
    entry.interpretations.i1 = "Sala, con \"comillas\"".to_string();

    let texto = String::from_utf8(build_csv(&entry)).expect("utf-8");

    assert!(texto.contains("\"Sala, con \"\"comillas\"\"\""));
}
