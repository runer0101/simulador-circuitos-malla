import logging
import os

from src.app_factory import create_app

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = create_app()


def _parse_env_bool(value: str) -> bool:
    return value.strip().lower() in {"1", "true", "yes", "on"}


if __name__ == "__main__":
    port = int(os.environ.get("PORT", 5000))
    debug_mode = _parse_env_bool(os.environ.get("FLASK_DEBUG", "False"))

    logger.info("Iniciando simulador-circuitos-malla...")
    logger.info(f"Puerto: {port}, Debug: {debug_mode}")

    app.run(host="0.0.0.0", port=port, debug=debug_mode)
