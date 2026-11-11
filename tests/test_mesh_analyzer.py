import pytest

from src.services.mesh_analyzer import MeshAnalyzer


def test_calcular_corrientes_returns_expected_shapes():
    I1, I2, I3, A, B = MeshAnalyzer.calcular_corrientes(
        R1=2.0,
        R2=4.0,
        R3=3.0,
        R4=6.0,
        R5=5.0,
        R6=2.0,
        V1=12.0,
        V2=0.0,
        V3=0.0,
    )

    assert isinstance(I1, float)
    assert isinstance(I2, float)
    assert isinstance(I3, float)
    assert A.shape == (3, 3)
    assert B.shape == (3,)


def test_calcular_corrientes_rejects_invalid_voltage_range():
    with pytest.raises(ValueError):
        MeshAnalyzer.calcular_corrientes(
            R1=2.0,
            R2=4.0,
            R3=3.0,
            R4=6.0,
            R5=5.0,
            R6=2.0,
            V1=501.0,
            V2=0.0,
            V3=0.0,
        )


def test_interpretar_corrientes_returns_all_meshes():
    result = MeshAnalyzer.interpretar_corrientes(0.5, -2.0, 12.0)

    assert set(result.keys()) == {"I1", "I2", "I3"}
    assert "Sala/Comedor" in result["I1"]


def test_clasificar_severidad_returns_expected_buckets():
    assert MeshAnalyzer.clasificar_severidad(0.5) == "baja"
    assert MeshAnalyzer.clasificar_severidad(-5.0) == "normal"
    assert MeshAnalyzer.clasificar_severidad(20.0) == "alta"
    assert MeshAnalyzer.clasificar_severidad(-100.0) == "critica"


def test_calcular_potencia_total_matches_manual_calculation():
    I1, I2, I3, _, _ = MeshAnalyzer.calcular_corrientes(
        R1=2.0,
        R2=4.0,
        R3=3.0,
        R4=6.0,
        R5=5.0,
        R6=2.0,
        V1=12.0,
        V2=0.0,
        V3=0.0,
    )
    params = {"R1": 2.0, "R2": 4.0, "R3": 3.0, "R4": 6.0, "R5": 5.0, "R6": 2.0}
    expected = (
        I1**2 * 2.0 + I2**2 * 4.0 + I3**2 * 3.0 + (I1 - I2) ** 2 * 6.0 + (I2 - I3) ** 2 * 5.0 + (I1 - I3) ** 2 * 2.0
    )
    assert abs(MeshAnalyzer.calcular_potencia_total(I1, I2, I3, params) - expected) < 1e-9
