import importlib.util
import json
import logging
import sys
from pathlib import Path

from src.logging_config import JsonFormatter, configure_logging


def _record(message="hola", level=logging.INFO, exc_info=None, **extra):
    record = logging.LogRecord("test", level, __file__, 10, message, None, exc_info)
    for key, value in extra.items():
        setattr(record, key, value)
    return record


def test_json_formatter_emits_one_parseable_object():
    payload = json.loads(JsonFormatter().format(_record("simulación completada")))

    assert payload["level"] == "INFO"
    assert payload["logger"] == "test"
    assert payload["message"] == "simulación completada"
    assert payload["time"]


def test_json_formatter_keeps_extra_fields():
    payload = json.loads(JsonFormatter().format(_record("x", route="/api/calculate")))

    assert payload["route"] == "/api/calculate"


def test_json_formatter_serialises_exceptions():
    try:
        raise ValueError("malla singular")
    except ValueError:
        exc_info = sys.exc_info()

    payload = json.loads(JsonFormatter().format(_record("fallo", exc_info=exc_info)))

    assert "ValueError" in payload["exception"]


def test_configure_logging_uses_json_when_requested(monkeypatch):
    monkeypatch.setenv("LOG_FORMAT", "json")
    monkeypatch.setenv("LOG_LEVEL", "warning")

    configure_logging()
    root = logging.getLogger()

    assert isinstance(root.handlers[0].formatter, JsonFormatter)
    assert root.level == logging.WARNING


def test_configure_logging_defaults_to_plain_text(monkeypatch):
    monkeypatch.delenv("LOG_FORMAT", raising=False)

    configure_logging()
    root = logging.getLogger()

    assert not isinstance(root.handlers[0].formatter, JsonFormatter)
    assert root.handlers[0].formatter._fmt == "%(asctime)s %(levelname)s %(name)s %(message)s"


def _load_gunicorn_conf():
    path = Path(__file__).resolve().parent.parent / "gunicorn.conf.py"
    spec = importlib.util.spec_from_file_location("gunicorn_conf", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_gunicorn_conf_reads_environment(monkeypatch):
    monkeypatch.setenv("PORT", "9999")
    monkeypatch.setenv("WEB_CONCURRENCY", "3")
    monkeypatch.setenv("LOG_LEVEL", "debug")

    conf = _load_gunicorn_conf()

    assert conf.bind == "0.0.0.0:9999"
    assert conf.workers == 3
    assert conf.loglevel == "debug"


def test_wsgi_entry_point_exposes_app():
    import wsgi

    assert wsgi.app is not None
    assert wsgi.app.test_client().get("/api/health").status_code == 200
