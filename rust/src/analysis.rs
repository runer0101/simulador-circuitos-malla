//! Interpretación de resultados: mismos textos, mismos umbrales y mismo redondeo que Python.

use crate::domain::MeshParams;

/// Monto a partir del cual la corriente se considera crítica.
pub const HIGH_CURRENT_WARNING_THRESHOLD: f64 = 1000.0;

pub const ZONAS: [(&str, &str); 3] = [
    ("I1", "Sala/Comedor"),
    ("I2", "Cocina/Lavandería"),
    ("I3", "Dormitorios"),
];

/// Describe cada corriente en el mismo formato que la versión Python.
pub fn interpretar_corrientes(corrientes: [f64; 3]) -> [String; 3] {
    let mut out = [String::new(), String::new(), String::new()];
    for (index, corriente) in corrientes.iter().enumerate() {
        let zona = ZONAS[index].1;
        let magnitud = corriente.abs();
        let sentido = if *corriente > 0.0 {
            "horario"
        } else {
            "antihorario"
        };
        out[index] = if magnitud < 0.001 {
            format!("{zona}: Corriente despreciable (~0A)")
        } else if magnitud < 1.0 {
            format!("{zona}: {magnitud:.3}A ({sentido}) - Carga baja")
        } else if magnitud < 10.0 {
            format!("{zona}: {magnitud:.2}A ({sentido}) - Carga normal")
        } else if magnitud < 50.0 {
            format!("{zona}: {magnitud:.1}A ({sentido}) - Carga alta")
        } else {
            format!("{zona}: {magnitud:.1}A ({sentido}) - [!] CARGA CRÍTICA")
        };
    }
    out
}

/// Severidad de una corriente, con los mismos cortes que la plantilla.
pub fn clasificar_severidad(corriente: f64) -> &'static str {
    let magnitud = corriente.abs();
    if magnitud < 1.0 {
        "baja"
    } else if magnitud < 10.0 {
        "normal"
    } else if magnitud < 50.0 {
        "alta"
    } else {
        "critica"
    }
}

/// Potencia total disipada, incluida la de las resistencias compartidas entre mallas.
pub fn potencia_total(corrientes: [f64; 3], params: &MeshParams) -> f64 {
    let [i1, i2, i3] = corrientes;
    i1 * i1 * params.r1
        + i2 * i2 * params.r2
        + i3 * i3 * params.r3
        + (i1 - i2).powi(2) * params.r4
        + (i2 - i3).powi(2) * params.r5
        + (i1 - i3).powi(2) * params.r6
}
