import logging
import os
from datetime import datetime, timezone

from flask import Blueprint, jsonify, request
from werkzeug.exceptions import BadRequest

from src.config import REQUIRED_PARAMS
from src.services.mesh_analyzer import MeshAnalyzer, get_example_values
from src.services.usage_metrics import usage_metrics
from src.validators.inputs import validate_api_payload

logger = logging.getLogger(__name__)
api_bp = Blueprint("api", __name__, url_prefix="/api")


def _api_error(status: int, code: str, message: str, details: str | None = None):
    payload = {
        "success": False,
        "error": {
            "code": code,
            "message": message,
            "details": details,
        },
    }
    return jsonify(payload), status


@api_bp.route("/calculate", methods=["POST"])
def api_calculate():
    try:
        if not request.is_json:
            return _api_error(
                415,
                "UNSUPPORTED_MEDIA_TYPE",
                "Content-Type debe ser application/json",
                "Envia la solicitud con header Content-Type: application/json",
            )

        data = request.get_json(silent=True)
        if data is None:
            return _api_error(400, "MALFORMED_JSON", "JSON malformado")

        params, error = validate_api_payload(data)
        if error:
            return _api_error(400, "INVALID_PAYLOAD", "Datos de entrada inválidos", error)

        R1, R2, R3, R4, R5, R6 = (params[key] for key in REQUIRED_PARAMS[:6])
        V1, V2, V3 = (params[key] for key in REQUIRED_PARAMS[6:])

        I1, I2, I3, A, B = MeshAnalyzer.calcular_corrientes(R1, R2, R3, R4, R5, R6, V1, V2, V3)
        interpretaciones = MeshAnalyzer.interpretar_corrientes(I1, I2, I3)

        return jsonify(
            {
                "success": True,
                "currents": {"I1": I1, "I2": I2, "I3": I3},
                "matrix_A": A.tolist(),
                "vector_B": B.tolist(),
                "interpretations": interpretaciones,
            }
        )
    except ValueError as exc:
        return _api_error(400, "CALCULATION_ERROR", "Error de cálculo", str(exc))
    except BadRequest:
        return _api_error(400, "MALFORMED_JSON", "JSON malformado")
    except Exception:
        logger.exception(
            "Error en API calculate",
            extra={"method": request.method, "path": request.path, "query": request.query_string.decode("utf-8")},
        )
        return _api_error(500, "INTERNAL_ERROR", "Error interno del servidor")


@api_bp.route("/example", methods=["GET"])
def api_example():
    return jsonify(get_example_values())


@api_bp.route("/health", methods=["GET"])
def api_health():
    return jsonify(
        {
            "status": "ok",
            "service": "simulacion_mallas",
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }
    )


@api_bp.route("/version", methods=["GET"])
def api_version():
    version = os.environ.get("APP_VERSION", "dev")
    return jsonify(
        {
            "service": "simulacion_mallas",
            "version": version,
        }
    )


@api_bp.route("/metrics", methods=["GET"])
def api_metrics():
    return jsonify(
        {
            "service": "simulacion_mallas",
            "metrics": usage_metrics.snapshot(),
        }
    )
