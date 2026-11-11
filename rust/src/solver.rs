//! Resolución del sistema 3x3 con eliminación gaussiana y pivoteo parcial, que es lo que hace
//! numpy por debajo: sin dependencias y con el mismo resultado dentro del error de redondeo.

/// Umbral por debajo del cual el sistema se considera singular. Igual que en la versión Python.
pub const SINGULAR_TOLERANCE: f64 = 1e-10;

pub const SINGULAR_MESSAGE: &str =
    "Sistema singular: Las resistencias crean un circuito indeterminado";

/// Determinante de una matriz 3x3.
pub fn determinante(a: [[f64; 3]; 3]) -> f64 {
    a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
}

/// Resuelve el sistema A x = b. Devuelve None cuando la matriz es singular.
pub fn resolver(a: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    if determinante(a).abs() < SINGULAR_TOLERANCE {
        return None;
    }

    let mut m = [[0.0_f64; 4]; 3];
    for (i, row) in a.iter().enumerate() {
        m[i][..3].copy_from_slice(row);
        m[i][3] = b[i];
    }

    for col in 0..3 {
        let mut pivot = col;
        for row in col + 1..3 {
            if m[row][col].abs() > m[pivot][col].abs() {
                pivot = row;
            }
        }
        m.swap(col, pivot);

        for row in col + 1..3 {
            let factor = m[row][col] / m[col][col];
            // El índice es necesario: se leen la fila pivote y la fila que se elimina a la vez.
            #[allow(clippy::needless_range_loop)]
            for k in col..4 {
                m[row][k] -= factor * m[col][k];
            }
        }
    }

    let mut x = [0.0_f64; 3];
    for row in (0..3).rev() {
        let mut sum = m[row][3];
        for k in row + 1..3 {
            sum -= m[row][k] * x[k];
        }
        x[row] = sum / m[row][row];
    }
    Some(x)
}
