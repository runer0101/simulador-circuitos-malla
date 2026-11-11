//! Lo que piden los enunciados de mallas: potencia en cada resistencia y comprobación de la LVK.

use crate::domain::MeshParams;

/// Corriente que atraviesa cada resistencia, en el orden R1..R6.
///
/// Las tres primeras son propias de una malla; las tres últimas son compartidas y llevan la
/// diferencia entre las corrientes de las dos mallas que unen.
pub fn corrientes_por_resistencia(corrientes: [f64; 3], _p: &MeshParams) -> [f64; 6] {
    let [i1, i2, i3] = corrientes;
    [i1, i2, i3, i1 - i2, i2 - i3, i1 - i3]
}

/// Potencia disipada en cada resistencia: P = I² · R.
pub fn potencia_por_resistencia(corrientes: [f64; 3], p: &MeshParams) -> [f64; 6] {
    let i = corrientes_por_resistencia(corrientes, p);
    let r = [p.r1, p.r2, p.r3, p.r4, p.r5, p.r6];
    [
        i[0] * i[0] * r[0],
        i[1] * i[1] * r[1],
        i[2] * i[2] * r[2],
        i[3] * i[3] * r[3],
        i[4] * i[4] * r[4],
        i[5] * i[5] * r[5],
    ]
}

/// Una línea de la comprobación: la suma de tensiones de una malla debe dar cero.
#[derive(Debug, Clone, Copy)]
pub struct VerificacionMalla {
    pub malla: usize,
    pub fuente: f64,
    pub caidas: f64,
    pub residuo: f64,
}

/// Comprueba la ley de tensiones de Kirchhoff en cada malla con las corrientes obtenidas.
///
/// Es la misma comprobación que se pide a mano en los trabajos: recorrer el lazo sumando
/// subidas y caídas y verificar que el total es cero.
pub fn verificar_lvk(corrientes: [f64; 3], p: &MeshParams) -> [VerificacionMalla; 3] {
    let [i1, i2, i3] = corrientes;
    let mallas = [
        (p.v1, p.r1 * i1 + p.r6 * (i1 - i3) + p.r4 * (i1 - i2)),
        (p.v2, p.r2 * i2 + p.r4 * (i2 - i1) + p.r5 * (i2 - i3)),
        (p.v3, p.r3 * i3 + p.r5 * (i3 - i2) + p.r6 * (i3 - i1)),
    ];
    let mut salida = [VerificacionMalla {
        malla: 1,
        fuente: 0.0,
        caidas: 0.0,
        residuo: 0.0,
    }; 3];
    for (indice, (fuente, caidas)) in mallas.into_iter().enumerate() {
        salida[indice] = VerificacionMalla {
            malla: indice + 1,
            fuente,
            caidas,
            residuo: fuente - caidas,
        };
    }
    salida
}
