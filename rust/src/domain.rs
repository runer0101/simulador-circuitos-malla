//! Parámetros del circuito y validación, con los mismos rangos y mensajes que la versión Python.

use std::collections::HashMap;
use std::fmt;

use serde::Serialize;

/// Rango admitido para una resistencia, en ohmios.
pub const RESISTANCE_RANGE: (f64, f64) = (0.01, 1000.0);
/// Rango admitido para una fuente de tensión, en voltios.
pub const VOLTAGE_RANGE: (f64, f64) = (0.0, 500.0);

pub const ERROR_INVALID_NUMBER: &str = "Debe ser un número válido";
pub const ERROR_POSITIVE_RESISTANCE: &str = "Debe ser un valor positivo";

/// Formatea un flotante como lo hace Python con f-string: 1000.0 y no 1000.
///
/// Importa para la paridad: el mensaje de error de rango interpola los extremos y el cliente ve el
/// texto tal cual.
pub fn format_float_py(value: f64) -> String {
    if value.fract() == 0.0 && value.is_finite() {
        format!("{value:.1}")
    } else {
        format!("{value}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationError {
    NotANumber { field: &'static str },
    NotPositive { field: &'static str },
    ResistanceOutOfRange { field: &'static str },
    VoltageOutOfRange { field: &'static str },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (min_r, max_r) = RESISTANCE_RANGE;
        let (min_v, max_v) = VOLTAGE_RANGE;
        match self {
            Self::NotANumber { field } => write!(f, "{field}: {ERROR_INVALID_NUMBER}"),
            Self::NotPositive { field } => write!(f, "{field}: {ERROR_POSITIVE_RESISTANCE}"),
            Self::ResistanceOutOfRange { field } => write!(
                f,
                "{field}: Resistencia debe estar entre {}Ω y {}Ω",
                format_float_py(min_r),
                format_float_py(max_r)
            ),
            Self::VoltageOutOfRange { field } => write!(
                f,
                "{field}: Voltaje debe estar entre {}V y {}V",
                format_float_py(min_v),
                format_float_py(max_v)
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Los nueve campos del circuito, en el mismo orden que usa la API.
pub const FIELD_KEYS: [&str; 9] = ["R1", "R2", "R3", "R4", "R5", "R6", "V1", "V2", "V3"];

/// Los nueve valores del circuito, en el mismo orden que usa la API.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
pub struct MeshParams {
    #[serde(rename = "R1")]
    pub r1: f64,
    #[serde(rename = "R2")]
    pub r2: f64,
    #[serde(rename = "R3")]
    pub r3: f64,
    #[serde(rename = "R4")]
    pub r4: f64,
    #[serde(rename = "R5")]
    pub r5: f64,
    #[serde(rename = "R6")]
    pub r6: f64,
    #[serde(rename = "V1")]
    pub v1: f64,
    #[serde(rename = "V2")]
    pub v2: f64,
    #[serde(rename = "V3")]
    pub v3: f64,
}

/// Valores por defecto del formulario, idénticos a los de la versión Python.
pub const DEFAULT_VALUES: MeshParams = MeshParams {
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

/// Valores de ejemplo de la instalación: los mismos que devuelve GET /api/example.
pub const EXAMPLE_VALUES: MeshParams = MeshParams {
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

impl MeshParams {
    /// Valida cada campo con el mismo criterio y en el mismo orden que la versión Python:
    /// números finitos, resistencias positivas y ambos rangos.
    pub fn validate(&self) -> Result<(), ValidationError> {
        for (field, value) in self.resistances() {
            if !value.is_finite() {
                return Err(ValidationError::NotANumber { field });
            }
            if value <= 0.0 {
                return Err(ValidationError::NotPositive { field });
            }
            if !(RESISTANCE_RANGE.0 <= value && value <= RESISTANCE_RANGE.1) {
                return Err(ValidationError::ResistanceOutOfRange { field });
            }
        }
        for (field, value) in self.voltages() {
            if !value.is_finite() {
                return Err(ValidationError::NotANumber { field });
            }
            if !(VOLTAGE_RANGE.0 <= value && value <= VOLTAGE_RANGE.1) {
                return Err(ValidationError::VoltageOutOfRange { field });
            }
        }
        Ok(())
    }

    fn resistances(&self) -> [(&'static str, f64); 6] {
        [
            ("R1", self.r1),
            ("R2", self.r2),
            ("R3", self.r3),
            ("R4", self.r4),
            ("R5", self.r5),
            ("R6", self.r6),
        ]
    }

    fn voltages(&self) -> [(&'static str, f64); 3] {
        [("V1", self.v1), ("V2", self.v2), ("V3", self.v3)]
    }

    /// Construye los parámetros a partir de los nueve valores en el orden de [`FIELD_KEYS`].
    pub fn from_values(values: [f64; 9]) -> Self {
        Self {
            r1: values[0],
            r2: values[1],
            r3: values[2],
            r4: values[3],
            r5: values[4],
            r6: values[5],
            v1: values[6],
            v2: values[7],
            v3: values[8],
        }
    }

    /// Asigna un campo por nombre. Devuelve `false` si el nombre no es uno de los nueve.
    pub fn set(&mut self, field: &str, value: f64) -> bool {
        match field {
            "R1" => self.r1 = value,
            "R2" => self.r2 = value,
            "R3" => self.r3 = value,
            "R4" => self.r4 = value,
            "R5" => self.r5 = value,
            "R6" => self.r6 = value,
            "V1" => self.v1 = value,
            "V2" => self.v2 = value,
            "V3" => self.v3 = value,
            _ => return false,
        }
        true
    }

    /// Matriz de resistencias del sistema de mallas (KVL), simétrica por construcción.
    ///
    /// La topología del circuito es la de tres mallas que se tocan dos a dos, y cada par comparte
    /// exactamente una rama:
    ///
    /// - `R4` une la malla 1 con la 2,
    /// - `R5` une la malla 2 con la 3,
    /// - `R6` une la malla 1 con la 3.
    ///
    /// Por eso la diagonal suma la resistencia propia de la malla y sus dos compartidas, y cada
    /// término fuera de la diagonal es la negativa de la compartida. Una escalera de tres ventanas
    /// en fila no puede representar esta matriz: allí la malla 1 y la 3 no comparten rama.
    pub fn matrix(&self) -> [[f64; 3]; 3] {
        [
            [self.r1 + self.r4 + self.r6, -self.r4, -self.r6],
            [-self.r4, self.r2 + self.r4 + self.r5, -self.r5],
            [-self.r6, -self.r5, self.r3 + self.r5 + self.r6],
        ]
    }

    /// Vector de fuentes de tensión.
    pub fn vector(&self) -> [f64; 3] {
        [self.v1, self.v2, self.v3]
    }
}

/// Convierte el texto de un campo con la única regla del proyecto: se recorta, se acepta coma
/// decimal y el error nombra el campo, igual que en la API, el formulario y la vista del PNG.
fn parse_field(field: &'static str, texto: &str) -> Result<f64, ValidationError> {
    texto
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| ValidationError::NotANumber { field })
}

/// Aplica sobre `base` los campos de texto presentes y no vacíos.
///
/// Los ausentes conservan el valor base, que es lo que hacen el formulario y la vista previa:
/// un campo oculto o vacío no debe romper el cálculo.
pub fn from_text_fields(
    base: MeshParams,
    campos: &HashMap<String, String>,
) -> Result<MeshParams, ValidationError> {
    let mut params = base;
    for field in FIELD_KEYS {
        if let Some(texto) = campos
            .get(field)
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            let value = parse_field(field, texto)?;
            let asignado = params.set(field, value);
            debug_assert!(asignado, "{field} pertenece a FIELD_KEYS");
        }
    }
    Ok(params)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// La rama compartida por cada par de mallas. Si la topología cambia, cambia esta tabla y el
    /// test exige que `matrix` la siga.
    const COMPARTIDAS: [(usize, usize, f64); 3] = [
        // (malla a, malla b, resistencia compartida)
        (0, 1, 6.0),
        (1, 2, 5.0),
        (0, 2, 2.0),
    ];

    fn ejemplo() -> MeshParams {
        EXAMPLE_VALUES
    }

    #[test]
    fn la_matriz_sigue_la_topologia_de_tres_mallas_que_se_tocan_dos_a_dos() {
        let p = ejemplo();
        let a = p.matrix();
        let propias = [p.r1, p.r2, p.r3];

        for (i, j, compartida) in COMPARTIDAS {
            assert_eq!(a[i][j], -compartida, "fuera de la diagonal ({i},{j})");
            assert_eq!(a[j][i], a[i][j], "la matriz debe ser simétrica ({j},{i})");
        }
        for (indice, propia) in propias.iter().enumerate() {
            let compartidas: f64 = COMPARTIDAS
                .iter()
                .filter(|(i, j, _)| *i == indice || *j == indice)
                .map(|(_, _, r)| *r)
                .sum();
            assert_eq!(a[indice][indice], propia + compartidas, "diagonal {indice}");
        }
    }

    /// Con los valores del ejemplo, la solución calculada a mano `[150, 84, 72]/71` tiene que
    /// cumplir las tres ecuaciones del sistema: así la matriz no sólo es simétrica, es la del
    /// circuito que se dibuja.
    #[test]
    fn la_solucion_calculada_a_mano_cumple_las_ecuaciones() {
        let p = ejemplo();
        let a = p.matrix();
        let b = p.vector();
        let corrientes = [150.0 / 71.0, 84.0 / 71.0, 72.0 / 71.0];

        for fila in 0..3 {
            let suma: f64 = (0..3).map(|col| a[fila][col] * corrientes[col]).sum();
            assert!(
                (suma - b[fila]).abs() < 1e-9,
                "fila {} da {suma} y debería dar {}",
                fila + 1,
                b[fila]
            );
        }
    }
}
