"""Logging setup shared by the dev server and the WSGI entry point.

Container deployments set LOG_FORMAT=json so every line is one parseable object.
"""

import json
import logging
import os
import sys

_RESERVED_ATTRS = frozenset(logging.makeLogRecord({}).__dict__) | {"message", "asctime"}

TEXT_FORMAT = "%(asctime)s %(levelname)s %(name)s %(message)s"
TIME_FORMAT = "%Y-%m-%dT%H:%M:%S%z"


class JsonFormatter(logging.Formatter):
    """Render one log record as a single-line JSON object."""

    def format(self, record: logging.LogRecord) -> str:
        payload = {
            "time": self.formatTime(record, TIME_FORMAT),
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
        }
        if record.exc_info:
            payload["exception"] = self.formatException(record.exc_info)
        for key, value in record.__dict__.items():
            if key not in _RESERVED_ATTRS and not key.startswith("_"):
                payload[key] = value
        return json.dumps(payload, ensure_ascii=False, default=str)


def safe_for_log(raw: bytes) -> str:
    """Keep untrusted text out of a log line: bound the length and drop line breaks."""
    return raw.decode("utf-8", "replace")[:200].replace("\n", " ").replace("\r", " ")


def configure_logging() -> None:
    """Send root logging to stdout, as JSON when LOG_FORMAT=json."""
    handler = logging.StreamHandler(sys.stdout)
    if os.environ.get("LOG_FORMAT", "").strip().lower() == "json":
        handler.setFormatter(JsonFormatter())
    else:
        handler.setFormatter(logging.Formatter(TEXT_FORMAT))

    root = logging.getLogger()
    root.handlers.clear()
    root.addHandler(handler)
    root.setLevel(os.environ.get("LOG_LEVEL", "INFO").strip().upper() or "INFO")
