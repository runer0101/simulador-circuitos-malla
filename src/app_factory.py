import logging
from pathlib import Path

from flask import Flask, render_template, request

from src.routes.api import api_bp
from src.routes.web import web_bp
from src.services.mesh_analyzer import get_default_values
from src.services.usage_metrics import usage_metrics

logger = logging.getLogger(__name__)


def create_app() -> Flask:
    project_root = Path(__file__).resolve().parent.parent
    app = Flask(
        __name__,
        template_folder=str(project_root / "templates"),
        static_folder=str(project_root / "static"),
    )
    app.register_blueprint(web_bp)
    app.register_blueprint(api_bp)
    app.before_request(_record_request_metrics)

    register_error_handlers(app)
    return app


def _record_request_metrics() -> None:
    usage_metrics.record_request(request.method, request.path)


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
