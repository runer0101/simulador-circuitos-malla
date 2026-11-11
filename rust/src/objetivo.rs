//! Pregunta típica de los enunciados: qué tensión haría que una corriente fuera cero.
//!
//! Con I₂ = 0, la ley de tensiones de la malla 2 se reduce a V₂ = a₂₁·I₁ + a₂₃·I₃. Las mallas 1 y 3
//! dan un sistema 2x2 del que salen I₁ e I₃.

use crate::domain::MeshParams;

/// Resultado del análisis inverso: la tensión que anula una corriente de malla.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Objetivo {
    pub malla: usize,
    pub tension: f64,
    pub corriente_1: f64,
    pub corriente_3: f64,
}

/// Resuelve qué tensión en la malla 2 deja I₂ = 0, manteniendo V₁ y V₃.
///
/// Con los valores del ejemplo (2, 4, 3, 6, 5 y 2 Ω; 12, 0 y 0 V) sale V₂ = -8,75 V, I₁ = 1,25 A e
/// I₃ = 0,25 A: al aplicar esa tensión, la malla 2 queda con 15·0 = 6·1,25 + 5·0,25 - 8,75 = 0. El
/// signo negativo no es un error: indica que la fuente debe invertirse, y el formulario no admite
/// tensiones negativas, así que ese valor es una respuesta del análisis y no una entrada.
///
/// Devuelve None si el sistema 2x2 es singular, es decir, si no hay una solución única.
pub fn tension_que_anula_i2(p: &MeshParams) -> Option<Objetivo> {
    let a = p.matrix();
    // Sistema con I2 = 0:
    //   a11·I1 + a13·I3 = V1
    //   a31·I1 + a33·I3 = V3
    let (a11, a13) = (a[0][0], a[0][2]);
    let (a31, a33) = (a[2][0], a[2][2]);
    let det = a11 * a33 - a13 * a31;
    // El determinante de este 2x2 va en Ω²: con resistencias del orden de 0,01 Ω el menor valor con
    // sentido es del orden de 1e-4, así que este umbral sólo descarta lo degenerado.
    if det.abs() < 1e-12 {
        return None;
    }
    let i1 = (p.v1 * a33 - a13 * p.v3) / det;
    let i3 = (a11 * p.v3 - a31 * p.v1) / det;
    // Y de la ley de tensiones de la malla 2, con I2 = 0:
    let v2 = a[1][0] * i1 + a[1][2] * i3;
    Some(Objetivo {
        malla: 2,
        tension: v2,
        corriente_1: i1,
        corriente_3: i3,
    })
}
