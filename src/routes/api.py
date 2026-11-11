import logging
import os
from datetime import datetime, timezone

from flask import Blueprint, Response, jsonify, request
from werkzeug.exceptions import BadRequest, RequestEntityTooLarge

from src.config import REQUIRED_PARAMS
from src.logging_config import safe_for_log
from src.services.exporter import build_csv, build_pdf
from src.services.mesh_analyzer import MeshAnalyzer, get_example_values
from src.services.scenario_compare import compare_scenarios
from src.services.simulation_store import simulation_store
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


def _resolve_currents(params):
    R1, R2, R3, R4, R5, R6 = (params[key] for key in REQUIRED_PARAMS[:6])
    V1, V2, V3 = (params[key] for key in REQUIRED_PARAMS[6:])
    I1, I2, I3, A, B = MeshAnalyzer.calcular_corrientes(R1, R2, R3, R4, R5, R6, V1, V2, V3)
    interpretations = MeshAnalyzer.interpretar_corrientes(I1, I2, I3)
    power_total = MeshAnalyzer.calcular_potencia_total(I1, I2, I3, params)
    currents = {"I1": I1, "I2": I2, "I3": I3}
    return params, currents, interpretations, power_total, A, B


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

        params, currents, interpretations, power_total, A, B = _resolve_currents(params)
        entry_id = simulation_store.add(params, currents, interpretations, power_total)

        return jsonify(
            {
                "success": True,
                "id": entry_id,
                "currents": currents,
                "matrix_A": A.tolist(),
                "vector_B": B.tolist(),
                "interpretations": interpretations,
                "power_total": power_total,
            }
        )
    except RequestEntityTooLarge:
        return _api_error(413, "PAYLOAD_TOO_LARGE", "La petición supera el tamaño permitido")
    except ValueError as exc:
        return _api_error(400, "CALCULATION_ERROR", "Error de cálculo", str(exc))
    except BadRequest:
        return _api_error(400, "MALFORMED_JSON", "JSON malformado")
    except Exception:
        logger.exception(
            "Error en API calculate",
            extra={"method": request.method, "path": request.path, "query": safe_for_log(request.query_string)},
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
            "service": "simulador-circuitos-malla",
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }
    )


@api_bp.route("/version", methods=["GET"])
def api_version():
    version = os.environ.get("APP_VERSION", "dev")
    return jsonify(
        {
            "service": "simulador-circuitos-malla",
            "version": version,
        }
    )


@api_bp.route("/metrics", methods=["GET"])
def api_metrics():
    return jsonify(
        {
            "service": "simulador-circuitos-malla",
            "metrics": usage_metrics.snapshot(),
            "history_size": simulation_store.size(),
            "history_capacity": simulation_store.capacity(),
        }
    )


@api_bp.route("/history", methods=["GET"])
def api_history_list():
    return jsonify(
        {
            "success": True,
            "count": simulation_store.size(),
            "capacity": simulation_store.capacity(),
            "items": simulation_store.list(),
        }
    )


@api_bp.route("/history/<entry_id>", methods=["GET"])
def api_history_get(entry_id: str):
    entry = simulation_store.get(entry_id)
    if entry is None:
        return _api_error(404, "NOT_FOUND", "Simulación no encontrada")
    return jsonify({"success": True, "item": entry})


@api_bp.route("/history", methods=["DELETE"])
def api_history_clear():
    removed = simulation_store.clear()
    return jsonify({"success": True, "removed": removed})


def _build_export_payload_from_request():
    if not request.is_json:
        return None, _api_error(
            415,
            "UNSUPPORTED_MEDIA_TYPE",
            "Content-Type debe ser application/json",
            "Envia la solicitud con header Content-Type: application/json",
        )

    data = request.get_json(silent=True)
    if data is None:
        return None, _api_error(400, "MALFORMED_JSON", "JSON malformado")

    if "id" in data and isinstance(data["id"], str):
        entry = simulation_store.get(data["id"])
        if entry is None:
            return None, _api_error(404, "NOT_FOUND", "Simulación no encontrada")
        return entry, None

    params, error = validate_api_payload(data)
    if error:
        return None, _api_error(400, "INVALID_PAYLOAD", "Datos de entrada inválidos", error)

    params, currents, interpretations, power_total, _, _ = _resolve_currents(params)
    return {
        "id": "ad-hoc",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "params": params,
        "currents": currents,
        "interpretations": interpretations,
        "power_total": power_total,
    }, None


@api_bp.route("/export/csv", methods=["POST"])
def api_export_csv():
    try:
        simulation, err = _build_export_payload_from_request()
        if err is not None:
            return err
        body = build_csv(simulation)
        filename = f"simulacion_{simulation.get('id', 'adhoc')}.csv"
        return Response(
            body,
            mimetype="text/csv; charset=utf-8",
            headers={"Content-Disposition": f'attachment; filename="{filename}"'},
        )
    except ValueError as exc:
        return _api_error(400, "CALCULATION_ERROR", "Error de cálculo", str(exc))
    except Exception:
        logger.exception("Error en API export csv")
        return _api_error(500, "INTERNAL_ERROR", "Error interno del servidor")


@api_bp.route("/export/pdf", methods=["POST"])
def api_export_pdf():
    try:
        simulation, err = _build_export_payload_from_request()
        if err is not None:
            return err
        body = build_pdf(simulation)
        filename = f"simulacion_{simulation.get('id', 'adhoc')}.pdf"
        return Response(
            body,
            mimetype="application/pdf",
            headers={"Content-Disposition": f'attachment; filename="{filename}"'},
        )
    except ValueError as exc:
        return _api_error(400, "CALCULATION_ERROR", "Error de cálculo", str(exc))
    except Exception:
        logger.exception("Error en API export pdf")
        return _api_error(500, "INTERNAL_ERROR", "Error interno del servidor")


@api_bp.route("/compare", methods=["POST"])
def api_compare():
    try:
        if not request.is_json:
            return _api_error(
                415,
                "UNSUPPORTED_MEDIA_TYPE",
                "Content-Type debe ser application/json",
            )
        data = request.get_json(silent=True)
        if data is None:
            return _api_error(400, "MALFORMED_JSON", "JSON malformado")
        if not isinstance(data, dict) or "baseline" not in data or "optimized" not in data:
            return _api_error(
                400,
                "INVALID_PAYLOAD",
                "Datos de entrada inválidos",
                "Se requieren los campos 'baseline' y 'optimized'",
            )

        baseline_params, err_a = validate_api_payload(data["baseline"])
        if err_a:
            return _api_error(400, "INVALID_PAYLOAD", "Datos de entrada inválidos (baseline)", err_a)
        optimized_params, err_b = validate_api_payload(data["optimized"])
        if err_b:
            return _api_error(400, "INVALID_PAYLOAD", "Datos de entrada inválidos (optimized)", err_b)

        baseline, optimized, comparison = compare_scenarios(baseline_params, optimized_params)
        return jsonify(
            {
                "success": True,
                "baseline": baseline,
                "optimized": optimized,
                "comparison": comparison,
            }
        )
    except ValueError as exc:
        return _api_error(400, "CALCULATION_ERROR", "Error de cálculo", str(exc))
    except Exception:
        logger.exception("Error en API compare")
        return _api_error(500, "INTERNAL_ERROR", "Error interno del servidor")


@api_bp.route("/report", methods=["GET"])
def api_report():
    aggregate = simulation_store.aggregate_report()
    return jsonify(
        {
            "service": "simulador-circuitos-malla",
            "report": aggregate,
            "history_capacity": simulation_store.capacity(),
        }
    )
