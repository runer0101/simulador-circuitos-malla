//! Núcleo del simulador de mallas, portado desde la implementación Python con paridad de
//! comportamiento: mismos rangos, mismos mensajes y mismas cifras.

pub mod analysis;
pub mod api;
pub mod domain;
pub mod exporter;
pub mod render;
pub mod solver;
pub mod web;

pub use analysis::{clasificar_severidad, interpretar_corrientes, potencia_total};
pub use domain::{MeshParams, ValidationError};
pub use solver::{determinante, resolver};
