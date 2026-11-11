//! Lo que piden los enunciados de mallas: potencia en cada resistencia y comprobación de la LVK.

use crate::domain::MeshParams;

/// Corriente que atraviesa cada resistencia, en el orden R1..R6.
///
/// Las tres primeras son propias de una malla; las tres últimas son compartidas y llevan la
/// diferencia entre las corrientes de las dos mallas que unen: R4 lleva I₁ - I₂, R5 lleva I₂ - I₃ y
/// R6 lleva I₁ - I₃. El signo sigue la convención de las tres mallas en sentido horario, así que el
/// valor es positivo cuando la corriente real va en el sentido de la malla que se nombra primero.
pub fn corrientes_por_resistencia(corrientes: [f64; 3], _p: &MeshParams) -> [f64; 6] {
    let [i1, i2, i3] = corrientes;
    [i1, i2, i3, i1 - i2, i2 - i3, i1 - i3]
}

/// Potencia disipada en cada resistencia: P = I² · R, con la corriente de cada rama y no la de la
/// malla (en las compartidas, la diferencia).
pub fn potencia_por_resistencia(corrientes: [f64; 3], p: &MeshParams) -> [f64; 6] {
    let i = corrientes_por_resistencia(corrientes, p);
    let r = [p.r1, p.r2, p.r3, p.r4, p.r5, p.r6];
    let mut potencias = [0.0_f64; 6];
    for (indice, corriente) in i.iter().enumerate() {
        potencias[indice] = corriente * corriente * r[indice];
    }
    potencias
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

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::domain::EXAMPLE_VALUES;

    /// La solución del ejemplo calculada a mano: I = [150, 84, 72]/71.
    fn corrientes_del_ejemplo() -> [f64; 3] {
        [150.0 / 71.0, 84.0 / 71.0, 72.0 / 71.0]
    }

    #[test]
    fn las_ramas_compartidas_llevan_la_diferencia_de_sus_dos_mallas() {
        let i = corrientes_por_resistencia(corrientes_del_ejemplo(), &EXAMPLE_VALUES);
        assert_eq!(i[3], 150.0 / 71.0 - 84.0 / 71.0, "R4 es I1 - I2");
        assert_eq!(i[4], 84.0 / 71.0 - 72.0 / 71.0, "R5 es I2 - I3");
        assert_eq!(i[5], 150.0 / 71.0 - 72.0 / 71.0, "R6 es I1 - I3");
    }

    /// P = I²R para cada rama, y la suma tiene que ser la potencia que entrega la única fuente
    /// activa (12 V · I₁): es la comprobación de conservación de la energía.
    #[test]
    fn la_potencia_de_cada_rama_es_i_cuadrado_por_r_y_suma_lo_que_entrega_la_fuente() {
        let p = EXAMPLE_VALUES;
        let corrientes = corrientes_del_ejemplo();
        let potencias = potencia_por_resistencia(corrientes, &p);
        let esperadas = [
            45000.0 / 5041.0,
            28224.0 / 5041.0,
            15552.0 / 5041.0,
            26136.0 / 5041.0,
            720.0 / 5041.0,
            12168.0 / 5041.0,
        ];
        for (indice, esperada) in esperadas.iter().enumerate() {
            assert!(
                (potencias[indice] - esperada).abs() < 1e-9,
                "P[{}] = {} y debería ser {esperada}",
                indice + 1,
                potencias[indice]
            );
        }
        let disipada: f64 = potencias.iter().sum();
        let entregada = p.v1 * corrientes[0] + p.v2 * corrientes[1] + p.v3 * corrientes[2];
        assert!(
            (disipada - entregada).abs() < 1e-9,
            "se disipan {disipada} W y la fuente entrega {entregada} W"
        );
    }

    #[test]
    fn la_ley_de_tensiones_cierra_en_las_tres_mallas() {
        let p = EXAMPLE_VALUES;
        let verificaciones = verificar_lvk(corrientes_del_ejemplo(), &p);
        for (indice, v) in verificaciones.iter().enumerate() {
            assert_eq!(v.malla, indice + 1);
            assert!(
                v.residuo.abs() < 1e-12,
                "malla {} no cierra: residuo {}",
                v.malla,
                v.residuo
            );
        }
        assert!((verificaciones[0].fuente - p.v1).abs() < 1e-12);
        assert!((verificaciones[0].caidas - p.v1).abs() < 1e-12);
    }
}
