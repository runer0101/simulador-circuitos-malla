//! Parámetros del circuito y validación, con los mismos rangos y mensajes que la versión Python.

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

    /// Matriz de resistencias del sistema de mallas (KVL), simétrica por construcción.
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
