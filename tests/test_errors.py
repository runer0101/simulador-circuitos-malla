"""Error branches of the API, the validators and the HTML routes.

Every test here pins a rejection path: the status code, the error contract and, where it matters,
that the rejected input left no trace in the history.
"""

import re
from pathlib import Path

import pytest

from src.app_factory import create_app
from src.config import ERROR_FORM_PARSE, EXAMPLE_VALUES
from src.services.rate_limit import api_rate_limiter
from src.validators.inputs import parse_form_data, validate_api_payload, validate_parameters


@pytest.fixture
def client():
    api_rate_limiter.reset()
    app = create_app()
    app.config["TESTING"] = True
    return app.test_client()


# --- history -------------------------------------------------------------------------------


def test_unknown_simulation_id_returns_404(client):
    response = client.get("/api/history/no-existe")

    assert response.status_code == 404
    assert response.get_json()["error"]["code"] == "NOT_FOUND"


def test_clearing_the_history_reports_how_many_entries_went(client):
    client.post("/api/calculate", json=EXAMPLE_VALUES)

    response = client.delete("/api/history")

    assert response.status_code == 200
    body = response.get_json()
    assert body["success"] is True
    assert body["removed"] == 1
    assert client.get("/api/history").get_json()["count"] == 0


# --- export --------------------------------------------------------------------------------


@pytest.mark.parametrize("route", ["/api/export/csv", "/api/export/pdf"])
def test_export_requires_json(client, route):
    response = client.post(route, data="R1=2")

    assert response.status_code == 415
    assert response.get_json()["error"]["code"] == "UNSUPPORTED_MEDIA_TYPE"


@pytest.mark.parametrize("route", ["/api/export/csv", "/api/export/pdf"])
def test_export_rejects_malformed_json(client, route):
    response = client.post(route, data="{no es json", content_type="application/json")

    assert response.status_code == 400
    assert response.get_json()["error"]["code"] == "MALFORMED_JSON"


@pytest.mark.parametrize("route", ["/api/export/csv", "/api/export/pdf"])
def test_export_rejects_unknown_id(client, route):
    response = client.post(route, json={"id": "no-existe"})

    assert response.status_code == 404
    assert response.get_json()["error"]["code"] == "NOT_FOUND"


def test_export_csv_renders_a_download(client):
    response = client.post("/api/export/csv", json=EXAMPLE_VALUES)

    assert response.status_code == 200
    assert response.mimetype == "text/csv"
    assert "attachment" in response.headers["Content-Disposition"]
    assert "ad-hoc" in response.headers["Content-Disposition"]
    assert b"25.352" in response.data or b"I1" in response.data


def test_export_pdf_renders_a_download(client):
    response = client.post("/api/export/pdf", json=EXAMPLE_VALUES)

    assert response.status_code == 200
    assert response.mimetype == "application/pdf"
    assert response.data.startswith(b"%PDF")


def test_export_by_stored_id_uses_that_id_in_the_filename(client):
    created = client.post("/api/calculate", json=EXAMPLE_VALUES).get_json()["id"]

    response = client.post("/api/export/csv", json={"id": created})

    assert response.status_code == 200
    assert created in response.headers["Content-Disposition"]


# --- compare -------------------------------------------------------------------------------


def test_compare_requires_json(client):
    response = client.post("/api/compare", data="baseline=1")

    assert response.status_code == 415


def test_compare_rejects_malformed_json(client):
    response = client.post("/api/compare", data="{", content_type="application/json")

    assert response.status_code == 400
    assert response.get_json()["error"]["code"] == "MALFORMED_JSON"


@pytest.mark.parametrize(
    "payload",
    [{}, {"baseline": EXAMPLE_VALUES}, {"optimized": EXAMPLE_VALUES}, ["no", "es", "dict"]],
)
def test_compare_requires_both_scenarios(client, payload):
    response = client.post("/api/compare", json=payload)

    assert response.status_code == 400
    assert response.get_json()["error"]["code"] == "INVALID_PAYLOAD"


def test_compare_names_the_invalid_scenario(client):
    response = client.post("/api/compare", json={"baseline": {"R1": 1}, "optimized": EXAMPLE_VALUES})

    assert response.status_code == 400
    details = response.get_json()["error"]["details"]
    assert "faltantes" in details.lower()


def test_compare_reports_which_scenario_failed_validation(client):
    bad = dict(EXAMPLE_VALUES, R1=0)
    response = client.post("/api/compare", json={"baseline": EXAMPLE_VALUES, "optimized": bad})

    assert response.status_code == 400
    body = response.get_json()
    assert body["error"]["code"] == "INVALID_PAYLOAD"
    assert "optimized" in body["error"]["message"]


# --- validators ----------------------------------------------------------------------------


@pytest.mark.parametrize("value", [float("nan"), float("inf"), "texto", None])
def test_non_numeric_values_are_rejected(value):
    with pytest.raises(ValueError) as excinfo:
        validate_parameters({"R1": value})

    assert "número válido" in str(excinfo.value)


def test_resistance_must_be_positive():
    with pytest.raises(ValueError) as excinfo:
        validate_parameters({"R1": 0})

    assert "positivo" in str(excinfo.value)


def test_resistance_must_stay_inside_its_range():
    with pytest.raises(ValueError) as excinfo:
        validate_parameters({"R1": 5000.0})

    assert "entre" in str(excinfo.value)


def test_voltage_must_stay_inside_its_range():
    with pytest.raises(ValueError) as excinfo:
        validate_parameters({"V1": 900.0})

    assert "entre" in str(excinfo.value)


def test_empty_payload_is_rejected():
    params, error = validate_api_payload({})

    assert params is None
    assert "No se recibieron datos" in error


def test_missing_parameters_are_named():
    params, error = validate_api_payload({"R1": 1})

    assert params is None
    assert "R1" not in error  # the report lists what is missing, not what arrived
    assert "R2" in error


def test_non_numeric_payload_reports_numeric_requirement():
    params, error = validate_api_payload(dict(EXAMPLE_VALUES, R1="dos"))

    assert params is None
    assert "numéricos" in error


def test_form_parse_falls_back_on_an_unexpected_structure():
    vals, error = parse_form_data(None, EXAMPLE_VALUES)

    assert error == ERROR_FORM_PARSE
    assert vals == EXAMPLE_VALUES


# --- unexpected failures -------------------------------------------------------------------


def _explode(*args, **kwargs):
    raise RuntimeError("fallo interno simulado")


def test_calculate_returns_the_contract_when_something_breaks(client, monkeypatch):
    from src.routes import api as api_module

    monkeypatch.setattr(api_module, "_resolve_currents", _explode)

    response = client.post("/api/calculate", json=EXAMPLE_VALUES)

    assert response.status_code == 500
    body = response.get_json()
    assert body["error"]["code"] == "INTERNAL_ERROR"
    assert "fallo interno" not in response.get_data(as_text=True)


@pytest.mark.parametrize("route,builder", [("/api/export/csv", "build_csv"), ("/api/export/pdf", "build_pdf")])
def test_export_returns_the_contract_when_something_breaks(client, monkeypatch, route, builder):
    from src.routes import api as api_module

    monkeypatch.setattr(api_module, builder, _explode)

    response = client.post(route, json=EXAMPLE_VALUES)

    assert response.status_code == 500
    assert response.get_json()["error"]["code"] == "INTERNAL_ERROR"


def test_compare_returns_the_contract_when_something_breaks(client, monkeypatch):
    from src.routes import api as api_module

    monkeypatch.setattr(api_module, "compare_scenarios", _explode)

    response = client.post("/api/compare", json={"baseline": EXAMPLE_VALUES, "optimized": EXAMPLE_VALUES})

    assert response.status_code == 500
    assert response.get_json()["error"]["code"] == "INTERNAL_ERROR"


def test_unknown_api_route_answers_json_instead_of_html(client):
    response = client.get("/api/no-existe")

    assert response.status_code == 404
    assert response.mimetype == "application/json"
    assert response.get_json()["error"]["code"] == "NOT_FOUND"


def test_unknown_page_still_renders_html(client):
    response = client.get("/no-existe")

    assert response.status_code == 404
    assert response.mimetype == "text/html"


# --- HTML routes ---------------------------------------------------------------------------


def test_form_with_an_invalid_value_renders_the_error_without_storing(client):
    response = client.post(
        "/",
        data="R1=0&R2=4&R3=3&R4=6&R5=5&R6=2&V1=12&V2=0&V3=0",
        content_type="application/x-www-form-urlencoded",
    )

    assert response.status_code == 200
    assert "positivo" in response.get_data(as_text=True).lower()
    assert client.get("/api/history").get_json()["count"] == 0


def test_circuit_png_rejects_a_non_numeric_count(client):
    response = client.get("/circuito.png?resistance_count=abc")

    assert response.status_code == 400


def test_circuit_png_clamps_out_of_range_counts(client):
    response = client.get("/circuito.png?resistance_count=99&voltage_count=99")

    assert response.status_code == 200
    assert response.mimetype == "image/png"


def test_form_reports_a_non_numeric_field_in_spanish(client):
    response = client.post("/", data="R1=abc&R2=4", content_type="application/x-www-form-urlencoded")
    body = response.get_data(as_text=True)

    assert "R1: Debe ser un número válido" in body
    assert "could not convert" not in body


def test_form_error_is_announced_to_screen_readers(client):
    response = client.post("/", data="R1=0", content_type="application/x-www-form-urlencoded")

    assert 'role="alert"' in response.get_data(as_text=True)


def test_high_severity_colour_keeps_enough_contrast():
    """WCAG AA asks 3:1 for large text and this value is rendered bold above 18px."""
    css = (Path(__file__).resolve().parent.parent / "static" / "styles.css").read_text()
    light = css.split('[data-theme="dark"]')[0]
    colour = re.search(r"--severity-high-color:\s*(#[0-9a-fA-F]{6})", light).group(1)

    def luminance(hex_colour):
        channels = [int(hex_colour[i : i + 2], 16) / 255 for i in (1, 3, 5)]
        linear = [c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4 for c in channels]
        return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]

    ratio = (1.05) / (luminance(colour) + 0.05)
    assert ratio >= 4.5, f"{colour} sólo alcanza {ratio:.2f}:1 sobre blanco"
