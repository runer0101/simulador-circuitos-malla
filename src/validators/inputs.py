from typing import Dict, Optional, Tuple

import numpy as np

from src.config import (
    ERROR_FORM_PARSE,
    ERROR_INVALID_NUMBER,
    ERROR_POSITIVE_RESISTANCE,
    REQUIRED_PARAMS,
    RESISTANCE_RANGE,
    VOLTAGE_RANGE,
)


def validate_parameters(params: Dict[str, float]) -> None:
    for key, value in params.items():
        if not isinstance(value, (int, float)) or np.isnan(value) or np.isinf(value):
            raise ValueError(f"{key}: {ERROR_INVALID_NUMBER}")

        if key.startswith("R"):
            if value <= 0:
                raise ValueError(f"{key}: {ERROR_POSITIVE_RESISTANCE}")
            min_val, max_val = RESISTANCE_RANGE
            if not (min_val <= value <= max_val):
                raise ValueError(f"{key}: Resistencia debe estar entre {min_val}Ω y {max_val}Ω")
        elif key.startswith("V"):
            min_val, max_val = VOLTAGE_RANGE
            if not (min_val <= value <= max_val):
                raise ValueError(f"{key}: Voltaje debe estar entre {min_val}V y {max_val}V")


def parse_form_data(form_data: Dict, default_vals: Dict[str, float]) -> Tuple[Dict[str, float], Optional[str]]:
    vals = default_vals.copy()

    try:
        for key in vals.keys():
            form_value = form_data.get(key, "").strip()
            if form_value:
                vals[key] = float(form_value.replace(",", "."))

        validate_parameters(vals)
        return vals, None
    except ValueError as exc:
        return vals, str(exc)
    except Exception:
        return vals, ERROR_FORM_PARSE


def validate_api_payload(data: Optional[Dict]) -> Tuple[Optional[Dict[str, float]], Optional[str]]:
    if not data:
        return None, "No se recibieron datos JSON"

    missing_params = [param for param in REQUIRED_PARAMS if param not in data]
    if missing_params:
        return None, f"Parámetros faltantes: {missing_params}"

    try:
        params = {key: float(data[key]) for key in REQUIRED_PARAMS}
    except (TypeError, ValueError):
        return None, "Los parámetros deben ser numéricos"

    try:
        validate_parameters(params)
    except ValueError as exc:
        # A value out of range is bad input, not a failed calculation: report it as such.
        return None, str(exc)
    return params, None
