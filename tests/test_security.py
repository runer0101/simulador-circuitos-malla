import inspect

import app as app_module
from src.app_factory import MAX_CONTENT_LENGTH, create_app
from src.services.rate_limit import api_rate_limiter


def _client(limit=None):
    api_rate_limiter.reset()
    app = create_app()
    app.config["TESTING"] = True
    if limit is not None:
        app.config["RATE_LIMIT_REQUESTS"] = limit
    return app.test_client()


def _script_src(csp):
    return [part.strip() for part in csp.split(";") if part.strip().startswith("script-src")][0]


def test_security_headers_are_present():
    headers = _client().get("/").headers

    assert headers["X-Content-Type-Options"] == "nosniff"
    assert headers["X-Frame-Options"] == "DENY"
    assert headers["Referrer-Policy"] == "same-origin"


def test_csp_forbids_objects_and_framing():
    csp = _client().get("/").headers["Content-Security-Policy"]

    assert "object-src 'none'" in csp
    assert "frame-ancestors 'none'" in csp
    assert "base-uri 'self'" in csp


def test_csp_allows_no_inline_scripts():
    csp = _client().get("/").headers["Content-Security-Policy"]

    assert "'unsafe-inline'" not in _script_src(csp)


def test_compromised_polyfill_domain_is_gone():
    body = _client().get("/").get_data(as_text=True)

    assert "polyfill.io" not in body


def test_oversized_api_payload_is_rejected_with_the_error_contract():
    client = _client()
    oversized = "x" * (MAX_CONTENT_LENGTH + 1)

    response = client.post("/api/calculate", data=oversized, content_type="application/json")

    assert response.status_code == 413
    body = response.get_json()
    assert body["success"] is False
    assert body["error"]["code"] == "PAYLOAD_TOO_LARGE"


def test_oversized_form_payload_renders_the_error_page():
    client = _client()
    oversized = "x" * (MAX_CONTENT_LENGTH + 1)

    response = client.post("/", data=oversized, content_type="application/x-www-form-urlencoded")

    assert response.status_code == 413
    assert "413" in response.get_data(as_text=True)


def test_circuit_png_is_throttled_because_it_renders():
    client = _client(limit=1)

    assert client.get("/circuito.png").status_code == 200
    assert client.get("/circuito.png").status_code == 429


def test_dev_server_binds_to_loopback_by_default():
    source = inspect.getsource(app_module)

    assert 'os.environ.get("HOST", "127.0.0.1")' in source
    assert 'host="0.0.0.0"' not in source
