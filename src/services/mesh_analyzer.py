import logging
from typing import Dict, Tuple

import numpy as np

from src.config import (
    DEFAULT_VALUES,
    EXAMPLE_VALUES,
    HIGH_CURRENT_WARNING_THRESHOLD,
    SINGULAR_MATRIX_TOLERANCE,
)
from src.validators.inputs import validate_parameters

logger = logging.getLogger(__name__)


class MeshAnalyzer:
    """Clase para el análisis de circuitos de mallas residenciales."""

    @staticmethod
    def calcular_corrientes(
        R1: float,
        R2: float,
        R3: float,
        R4: float,
        R5: float,
        R6: float,
        V1: float,
        V2: float,
        V3: float,
    ) -> Tuple[float, float, float, np.ndarray, np.ndarray]:
        params = {
            "R1": R1,
            "R2": R2,
            "R3": R3,
            "R4": R4,
            "R5": R5,
            "R6": R6,
            "V1": V1,
            "V2": V2,
            "V3": V3,
        }
        validate_parameters(params)

        A = np.array(
            [
                [R1 + R4 + R6, -R4, -R6],
                [-R4, R2 + R4 + R5, -R5],
                [-R6, -R5, R3 + R5 + R6],
            ],
            dtype=np.float64,
        )
        B = np.array([V1, V2, V3], dtype=np.float64)

        det_A = np.linalg.det(A)
        if abs(det_A) < SINGULAR_MATRIX_TOLERANCE:
            raise ValueError("Sistema singular: Las resistencias crean un circuito indeterminado")

        try:
            currents = np.linalg.solve(A, B)
            max_current = max(abs(i) for i in currents)
            if max_current > HIGH_CURRENT_WARNING_THRESHOLD:
                logger.warning(f"Corriente muy alta detectada: {max_current:.2f}A")
            return float(currents[0]), float(currents[1]), float(currents[2]), A, B
        except np.linalg.LinAlgError as exc:
            raise ValueError(f"Error al resolver el sistema: {str(exc)}")
        except Exception as exc:
            raise ValueError(f"Error inesperado en el cálculo: {str(exc)}")

    @staticmethod
    def interpretar_corrientes(I1: float, I2: float, I3: float) -> Dict[str, str]:
        interpretaciones: Dict[str, str] = {}

        corrientes = {"I1": I1, "I2": I2, "I3": I3}
        zonas = {"I1": "Sala/Comedor", "I2": "Cocina/Lavandería", "I3": "Dormitorios"}

        for nombre, corriente in corrientes.items():
            magnitud = abs(corriente)
            sentido = "horario" if corriente > 0 else "antihorario"
            zona = zonas[nombre]

            if magnitud < 0.001:
                interpretaciones[nombre] = f"{zona}: Corriente despreciable (~0A)"
            elif magnitud < 1:
                interpretaciones[nombre] = f"{zona}: {magnitud:.3f}A ({sentido}) - Carga baja"
            elif magnitud < 10:
                interpretaciones[nombre] = f"{zona}: {magnitud:.2f}A ({sentido}) - Carga normal"
            elif magnitud < 50:
                interpretaciones[nombre] = f"{zona}: {magnitud:.1f}A ({sentido}) - Carga alta"
            else:
                interpretaciones[nombre] = f"{zona}: {magnitud:.1f}A ({sentido}) - [!] CARGA CRÍTICA"

        return interpretaciones


def get_default_values() -> Dict[str, float]:
    return DEFAULT_VALUES.copy()


def get_example_values() -> Dict[str, float]:
    return EXAMPLE_VALUES.copy()
