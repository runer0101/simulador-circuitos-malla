//! Interpretación de resultados: mismos textos, mismos umbrales y mismo redondeo que Python.

use crate::domain::MeshParams;

/// Franjas de magnitud de la corriente de una malla, en amperios.
///
/// No son un límite de ampacidad: el modelo no conoce la sección del conductor, así que la app no
/// puede decidir si un cable concreto se calienta. Sirven para ordenar el resultado. La referencia
/// real de una vivienda son los interruptores automáticos de sus circuitos derivados (REBT
/// ITC-BT-25 y equivalentes IEC: 10 A alumbrado, 16 A tomas de uso general, 25 A cocina), de modo
/// que 25 A es ya el techo de cualquier circuito doméstico. Por eso la franja crítica empieza en
/// 50 A, el doble del mayor calibre doméstico: es el punto en el que el valor no admite ninguna
/// duda, porque ninguna vivienda alimenta una zona con esa corriente.
pub const LOW_CURRENT_MAX: f64 = 1.0;
/// Final de la franja «normal»: por encima empieza la carga alta.
pub const NORMAL_CURRENT_MAX: f64 = 10.0;
/// Final de la franja «alta»: a partir de aquí la corriente se considera crítica.
pub const HIGH_CURRENT_MAX: f64 = 50.0;

/// Monto a partir del cual la corriente se considera crítica. Es el final de la franja alta: el
/// valor anterior (1000 A) no describía ninguna decisión del programa.
pub const HIGH_CURRENT_WARNING_THRESHOLD: f64 = HIGH_CURRENT_MAX;

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
        } else if magnitud < LOW_CURRENT_MAX {
            format!("{zona}: {magnitud:.3}A ({sentido}) - Carga baja")
        } else if magnitud < NORMAL_CURRENT_MAX {
            format!("{zona}: {magnitud:.2}A ({sentido}) - Carga normal")
        } else if magnitud < HIGH_CURRENT_MAX {
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
    if magnitud < LOW_CURRENT_MAX {
        "baja"
    } else if magnitud < NORMAL_CURRENT_MAX {
        "normal"
    } else if magnitud < HIGH_CURRENT_MAX {
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

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Las dos salidas de la severidad (el texto de la API y la clase que colorea la plantilla)
    /// tienen que cortar en el mismo punto, y el umbral publicado tiene que ser ese punto.
    #[test]
    fn la_franja_critica_empieza_donde_termina_la_alta() {
        assert_eq!(
            HIGH_CURRENT_WARNING_THRESHOLD, HIGH_CURRENT_MAX,
            "el umbral publicado no describe la franja crítica"
        );
        assert_eq!(clasificar_severidad(HIGH_CURRENT_MAX - 0.001), "alta");
        assert_eq!(clasificar_severidad(HIGH_CURRENT_MAX), "critica");
        assert_eq!(clasificar_severidad(NORMAL_CURRENT_MAX - 0.001), "normal");
        assert_eq!(clasificar_severidad(NORMAL_CURRENT_MAX), "alta");
        assert_eq!(clasificar_severidad(LOW_CURRENT_MAX), "normal");
    }

    /// Documenta la escala: 25 A es el techo de cualquier circuito derivado de una vivienda y la app
    /// todavía lo llama «carga alta»; la franja crítica empieza en el doble de ese techo.
    #[test]
    fn la_carga_critica_queda_por_encima_de_cualquier_circuito_domestico() {
        const MAYOR_CIRCUITO_DOMESTICO: f64 = 25.0;
        assert_eq!(
            clasificar_severidad(MAYOR_CIRCUITO_DOMESTICO),
            "alta",
            "25 A sigue siendo una franja que la app no marca como crítica"
        );
        assert_eq!(
            clasificar_severidad(2.0 * MAYOR_CIRCUITO_DOMESTICO),
            "critica",
            "la franja crítica debería empezar en el doble del mayor calibre doméstico"
        );
        assert_eq!(
            interpretar_corrientes([30.0, 8.0, 0.5])[0],
            "Sala/Comedor: 30.0A (horario) - Carga alta"
        );
        assert_eq!(
            interpretar_corrientes([30.0, 8.0, 0.5])[1],
            "Cocina/Lavandería: 8.00A (horario) - Carga normal"
        );
        assert_eq!(
            interpretar_corrientes([30.0, 8.0, 0.5])[2],
            "Dormitorios: 0.500A (horario) - Carga baja"
        );
        assert_eq!(
            interpretar_corrientes([-60.0, 0.0, 0.0])[0],
            "Sala/Comedor: 60.0A (antihorario) - [!] CARGA CRÍTICA"
        );
    }
}
