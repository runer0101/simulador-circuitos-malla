//! Exportación de una simulación a CSV. El formato replica el del exportador Python.

use crate::api::Entry;

/// Escapa un campo como lo hace el módulo csv de Python: comillas dobles si contiene coma,
/// comilla o salto de línea, y las comillas internas se duplican.
fn campo(valor: &str) -> String {
    let necesita =
        valor.contains(',') || valor.contains('"') || valor.contains('\n') || valor.contains('\r');
    if necesita {
        format!("\"{}\"", valor.replace('"', "\"\""))
    } else {
        valor.to_string()
    }
}

fn fila(campos: &[&str]) -> String {
    // Python escribe CRLF por defecto; el archivo debe salir idéntico.
    campos
        .iter()
        .map(|c| campo(c))
        .collect::<Vec<_>>()
        .join(",")
        + "\r\n"
}

/// Construye el CSV completo de una simulación.
pub fn build_csv(entry: &Entry) -> Vec<u8> {
    let mut out = String::new();
    out.push_str(&fila(&["simulador-circuitos-malla"]));
    out.push_str(&fila(&["id", &entry.id]));
    out.push_str(&fila(&["timestamp", &entry.timestamp]));
    out.push_str(&fila(&[]));

    out.push_str(&fila(&["Malla", "Corriente (A)", "Interpretación"]));
    for (indice, clave) in ["I1", "I2", "I3"].iter().enumerate() {
        let corriente = format!("{:.6}", entry.currents.array()[indice]);
        out.push_str(&fila(&[
            clave,
            &corriente,
            &entry.interpretations_array()[indice],
        ]));
    }
    out.push_str(&fila(&[
        "Potencia total (W)",
        &format!("{:.4}", entry.power_total),
        "",
    ]));
    out.push_str(&fila(&["--- Parámetros ---", "", ""]));
    for (clave, valor) in [
        ("R1", entry.params.r1),
        ("R2", entry.params.r2),
        ("R3", entry.params.r3),
        ("R4", entry.params.r4),
        ("R5", entry.params.r5),
        ("R6", entry.params.r6),
        ("V1", entry.params.v1),
        ("V2", entry.params.v2),
        ("V3", entry.params.v3),
    ] {
        out.push_str(&fila(&[clave, &format!("{valor:.4}"), ""]));
    }
    out.into_bytes()
}
