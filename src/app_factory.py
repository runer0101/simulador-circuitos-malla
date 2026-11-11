import logging
from pathlib import Path

from flask import Flask, current_app, jsonify, render_template, request

from src.config import RATE_LIMIT_REQUESTS
from src.routes.api import api_bp
from src.routes.web import web_bp
from src.services.mesh_analyzer import get_default_values
from src.services.rate_limit import api_rate_limiter
from src.services.usage_metrics import usage_metrics

logger = logging.getLogger(__name__)


def create_app() -> Flask:
    project_root = Path(__file__).resolve().parent.parent
    app = Flask(
        __name__,
        template_folder=str(project_root / "templates"),
        static_folder=str(project_root / "static"),
    )
    app.config["RATE_LIMIT_REQUESTS"] = RATE_LIMIT_REQUESTS
    app.register_blueprint(web_bp)
    app.register_blueprint(api_bp)
    app.before_request(_record_request_metrics)
    app.before_request(_enforce_rate_limit)

    register_error_handlers(app)
    return app


def _record_request_metrics() -> None:
    usage_metrics.record_request(request.method, request.path)


def _enforce_rate_limit():
    """Throttle the expensive POST endpoints; reads stay unlimited."""
    if request.method != "POST" or not request.path.startswith("/api/"):
        return None

    limit = current_app.config["RATE_LIMIT_REQUESTS"]
    if limit <= 0:
        return None

    allowed, retry_after = api_rate_limiter.check(request.remote_addr or "desconocido", limit)
    if allowed:
        return None

    payload = {
        "success": False,
        "error": {
            "code": "rate_limited",
            "message": "Demasiadas peticiones seguidas. Espere antes de volver a intentarlo.",
            "details": f"límite de {limit} peticiones cada {api_rate_limiter.window:g} s",
        },
    }
    response = jsonify(payload)
    response.status_code = 429
    response.headers["Retry-After"] = str(retry_after)
    return response


def register_error_handlers(app: Flask) -> None:
    @app.errorhandler(404)
    def not_found(error):
        template_data = {
            "vals": get_default_values(),
            "error": "Página no encontrada (Error 404)",
            "I1": None,
            "I2": None,
            "I3": None,
            "A": None,
            "B": None,
            "interpretaciones": None,
            "default_vals": get_default_values(),
        }
        return render_template("index.html", **template_data), 404

    @app.errorhandler(500)
    def internal_error(error):
        logger.error(
            "Error interno no controlado: %s",
            error,
            extra={"method": request.method, "path": request.path, "query": request.query_string.decode("utf-8")},
        )
        template_data = {
            "vals": get_default_values(),
            "error": "Error interno del servidor (Error 500). Revisa la consola para más detalles.",
            "I1": None,
            "I2": None,
            "I3": None,
            "A": None,
            "B": None,
            "interpretaciones": None,
            "default_vals": get_default_values(),
        }
        return render_template("index.html", **template_data), 500
