//! La tensión que anula I₂ debe anularla de verdad al resolver el circuito.

use simulador_circuitos_malla::domain::MeshParams;
use simulador_circuitos_malla::objetivo::tension_que_anula_i2;
use simulador_circuitos_malla::resolver;

fn ejemplo() -> MeshParams {
    MeshParams {
        r1: 2.0,
        r2: 4.0,
        r3: 3.0,
        r4: 6.0,
        r5: 5.0,
        r6: 2.0,
        v1: 12.0,
        v2: 0.0,
        v3: 0.0,
    }
}

#[test]
fn la_tension_calculada_deja_i2_en_cero() {
    let mut p = ejemplo();
    let objetivo = tension_que_anula_i2(&p).expect("hay solucion");

    // Se aplica esa tensión y se resuelve el circuito completo.
    p.v2 = objetivo.tension;
    let x = resolver(p.matrix(), p.vector()).expect("sistema con solucion");

    assert!(x[1].abs() < 1e-9, "I2 deberia ser cero y es {}", x[1]);
    assert!((x[0] - objetivo.corriente_1).abs() < 1e-9);
    assert!((x[2] - objetivo.corriente_3).abs() < 1e-9);
}

#[test]
fn con_las_fuentes_del_ejemplo_la_tension_sale_negativa() {
    // V1 = 12 V y V3 = 0: para dejar I2 en cero hay que aplicar -8.75 V, es decir, invertir la
    // fuente de esa malla. El signo no es un error: indica la polaridad necesaria.
    let objetivo = tension_que_anula_i2(&ejemplo()).expect("hay solucion");
    assert!(
        (objetivo.tension - (-8.75)).abs() < 1e-9,
        "tension: {}",
        objetivo.tension
    );
}

#[test]
fn sin_fuentes_no_hay_tension_que_anule_nada() {
    let p = MeshParams {
        v1: 0.0,
        v2: 0.0,
        v3: 0.0,
        ..ejemplo()
    };
    let objetivo = tension_que_anula_i2(&p).expect("sistema no singular");
    assert!(objetivo.tension.abs() < 1e-12);
}
