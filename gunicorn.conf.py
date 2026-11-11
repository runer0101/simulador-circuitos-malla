"""Gunicorn settings, overridable through environment variables."""

import os

bind = f"0.0.0.0:{os.environ.get('PORT', '8000')}"
workers = int(os.environ.get("WEB_CONCURRENCY", "2"))
timeout = int(os.environ.get("WEB_TIMEOUT", "60"))
accesslog = "-"
errorlog = "-"
loglevel = os.environ.get("LOG_LEVEL", "INFO").strip().lower() or "info"
