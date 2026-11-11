"""WSGI entry point used by Gunicorn: `gunicorn --config gunicorn.conf.py wsgi:app`."""

from src.app_factory import create_app
from src.logging_config import configure_logging

configure_logging()

app = create_app()
