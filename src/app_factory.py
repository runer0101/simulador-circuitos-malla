import logging
from pathlib import Path

from flask import Flask, current_app, jsonify, render_template, request

from src.config import RATE_LIMIT_REQUESTS
from src.logging_config import safe_for_log
from src.routes.api import api_bp
from src.routes.web import web_bp
from src.services.mesh_analyzer import get_default_values
from src.services.rate_limit import api_rate_limiter
from src.services.usage_metrics import usage_metrics

logger = logging.getLogger(__name__)

# 64 KB is far more than the nine numbers the form or the JSON payload can carry.
MAX_CONTENT_LENGTH = 64 * 1024

# MathJax comes from jsDelivr; everything else is served from this origin. Inline styles stay
# allowed because the template uses style attributes and MathJax writes its own.
SECURITY_HEADERS = {
    "Content-Security-Policy": (
        "default-src 'self'; "
        "script-src 'self' https://cdn.jsdelivr.net; "
        "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; "
        "img-src 'self' data:; "
        "font-src 'self' data: https://cdn.jsdelivr.net https://fonts.gstatic.com; "
        "connect-src 'self'; "
        "object-src 'none'; "
        "base-uri 'self'; "
        "form-action 'self'; "
        "frame-ancestors 'none'"
    ),
    "X-Content-Type-Options": "nosniff",
    "X-Frame-Options": "DENY",
    "Referrer-Policy": "same-origin",
}


def create_app() -> Flask:
    project_root = Path(__file__).resolve().parent.parent
    app = Flask(
        __name__,
        template_folder=str(project_root / "templates"),
        static_folder=str(project_root / "static"),
    )
    app.config["RATE_LIMIT_REQUESTS"] = RATE_LIMIT_REQUESTS
    app.config["MAX_CONTENT_LENGTH"] = MAX_CONTENT_LENGTH
    app.register_blueprint(web_bp)
    app.register_blueprint(api_bp)
    app.before_request(_record_request_metrics)
    app.before_request(_enforce_rate_limit)
    app.after_request(_apply_security_headers)

    register_error_handlers(app)
    return app


def _record_request_metrics() -> None:
    usage_metrics.record_request(request.method, request.path)


def _apply_security_headers(response):
    for name, value in SECURITY_HEADERS.items():
        response.headers.setdefault(name, value)
    return response


def _is_costly_request() -> bool:
    """The JSON API and the HTML form run the same matrix solve; the PNG costs a render."""
    if request.path == "/circuito.png":
        return True
    return request.method == "POST" and (request.path.startswith("/api/") or request.path == "/")


def _enforce_rate_limit():
    """Throttle the expensive POST endpoints; reads stay unlimited."""
    if not _is_costly_request():
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
            "code": "RATE_LIMITED",
            "message": "Demasiadas peticiones seguidas. Espere antes de volver a intentarlo.",
            "details": f"límite de {limit} peticiones cada {api_rate_limiter.window:g} s",
        },
    }
    response = jsonify(payload)
    response.status_code = 429
    response.headers["Retry-After"] = str(retry_after)
    return response


def _error_page(message: str) -> dict:
    return {
        "vals": get_default_values(),
        "error": message,
        "I1": None,
        "I2": None,
        "I3": None,
        "A": None,
        "B": None,
        "interpretaciones": None,
        "default_vals": get_default_values(),
    }


def register_error_handlers(app: Flask) -> None:
    def api_error_response(status: int, code: str, message: str):
        """The API answers in JSON even when the failure happened outside a handler."""
        payload = {"success": False, "error": {"code": code, "message": message, "details": None}}
        return jsonify(payload), status

    @app.errorhandler(404)
    def not_found(error):
        if request.path.startswith("/api/"):
            return api_error_response(404, "NOT_FOUND", "Recurso no encontrado")
        return render_template("index.html", **_error_page("Página no encontrada (Error 404)")), 404

    @app.errorhandler(413)
    def payload_too_large(error):
        limit = app.config["MAX_CONTENT_LENGTH"]
        if request.path.startswith("/api/"):
            payload = {
                "success": False,
                "error": {
                    "code": "PAYLOAD_TOO_LARGE",
                    "message": "La petición supera el tamaño permitido.",
                    "details": f"máximo {limit} bytes",
                },
            }
            return jsonify(payload), 413
        return render_template("index.html", **_error_page("La petición es demasiado grande (Error 413).")), 413

    @app.errorhandler(500)
    def internal_error(error):
        logger.error(
            "Error interno no controlado: %s",
            error,
            extra={"method": request.method, "path": request.path, "query": safe_for_log(request.query_string)},
        )
        if request.path.startswith("/api/"):
            return api_error_response(500, "INTERNAL_ERROR", "Error interno del servidor")
        template_data = _error_page("Error interno del servidor (Error 500). Revisa la consola para más detalles.")
        return render_template("index.html", **template_data), 500
