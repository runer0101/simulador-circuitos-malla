import os
from typing import Dict, Tuple

RATE_LIMIT_REQUESTS = int(os.environ.get("RATE_LIMIT_REQUESTS", "60"))

RESISTANCE_RANGE: Tuple[float, float] = (0.01, 1000.0)
VOLTAGE_RANGE: Tuple[float, float] = (0.0, 500.0)

REQUIRED_PARAMS = ["R1", "R2", "R3", "R4", "R5", "R6", "V1", "V2", "V3"]

DEFAULT_VALUES: Dict[str, float] = {
    "R1": 0.5,
    "R2": 0.7,
    "R3": 0.6,
    "R4": 20.0,
    "R5": 15.0,
    "R6": 25.0,
    "V1": 120.0,
    "V2": 220.0,
    "V3": 120.0,
}

EXAMPLE_VALUES: Dict[str, float] = {
    "R1": 2.0,
    "R2": 4.0,
    "R3": 3.0,
    "R4": 6.0,
    "R5": 5.0,
    "R6": 2.0,
    "V1": 12.0,
    "V2": 0.0,
    "V3": 0.0,
}

SINGULAR_MATRIX_TOLERANCE = 1e-10
HIGH_CURRENT_WARNING_THRESHOLD = 1000.0

ERROR_INVALID_NUMBER = "Debe ser un número válido"
ERROR_POSITIVE_RESISTANCE = "Debe ser un valor positivo"
ERROR_FORM_PARSE = "Error al procesar los datos. Verifica el formato de los números."
