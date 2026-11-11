"""Servicio de comparación de escenarios (antes/después) para análisis de optimización."""

from __future__ import annotations

from typing import Any, Dict, Tuple

from src.services.mesh_analyzer import MeshAnalyzer


def _build_simulation_result(params: Dict[str, float]) -> Dict[str, Any]:
    I1, I2, I3, _, _ = MeshAnalyzer.calcular_corrientes(
        params["R1"],
        params["R2"],
        params["R3"],
        params["R4"],
        params["R5"],
        params["R6"],
        params["V1"],
        params["V2"],
        params["V3"],
    )
    interpretations = MeshAnalyzer.interpretar_corrientes(I1, I2, I3)
    power = MeshAnalyzer.calcular_potencia_total(I1, I2, I3, params)
    return {
        "params": {k: float(v) for k, v in params.items()},
        "currents": {"I1": I1, "I2": I2, "I3": I3},
        "interpretations": interpretations,
        "power_total": float(power),
        "max_current": float(max(abs(I1), abs(I2), abs(I3))),
    }


def compare_scenarios(
    baseline: Dict[str, float], optimized: Dict[str, float]
) -> Tuple[Dict[str, Any], Dict[str, Any], Dict[str, Any]]:
    base = _build_simulation_result(baseline)
    opt = _build_simulation_result(optimized)

    def diff(key: str) -> float:
        return float(opt[key] - base[key])

    def pct(key: str) -> float | None:
        if base[key] == 0:
            return None
        return float((opt[key] - base[key]) / base[key] * 100.0)

    base_currents = base["currents"]
    opt_currents = opt["currents"]
    current_diffs = {
        f"I{idx}": {"absolute": diff_curr, "percent": pct_curr}
        for idx, (diff_curr, pct_curr) in enumerate(
            zip(
                [
                    opt_currents["I1"] - base_currents["I1"],
                    opt_currents["I2"] - base_currents["I2"],
                    opt_currents["I3"] - base_currents["I3"],
                ],
                [
                    (
                        None
                        if base_currents["I1"] == 0
                        else (opt_currents["I1"] - base_currents["I1"]) / base_currents["I1"] * 100.0
                    ),
                    (
                        None
                        if base_currents["I2"] == 0
                        else (opt_currents["I2"] - base_currents["I2"]) / base_currents["I2"] * 100.0
                    ),
                    (
                        None
                        if base_currents["I3"] == 0
                        else (opt_currents["I3"] - base_currents["I3"]) / base_currents["I3"] * 100.0
                    ),
                ],
            ),
            start=1,
        )
    }

    deltas: Dict[str, Any] = {
        **current_diffs,
        "power_total": {"absolute": diff("power_total"), "percent": pct("power_total")},
        "max_current": {"absolute": diff("max_current"), "percent": pct("max_current")},
    }

    power_reduction_pct = pct("power_total")
    reduction_label = None
    if power_reduction_pct is not None:
        if power_reduction_pct < 0:
            reduction_label = "mejora"
        elif power_reduction_pct > 0:
            reduction_label = "empeoramiento"
        else:
            reduction_label = "sin cambio"

    return (
        base,
        opt,
        {
            "deltas": deltas,
            "power_reduction_label": reduction_label,
            "baseline_max_severity": MeshAnalyzer.clasificar_severidad(base["max_current"]),
            "optimized_max_severity": MeshAnalyzer.clasificar_severidad(opt["max_current"]),
        },
    )
