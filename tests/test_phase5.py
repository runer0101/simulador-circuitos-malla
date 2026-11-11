import csv
import io
import json

import pytest

from src.app_factory import create_app
from src.config import DEFAULT_VALUES
from src.services.exporter import build_csv, build_pdf
from src.services.simulation_store import SimulationStore


@pytest.fixture
def client():
    app = create_app()
    app.config["TESTING"] = True
    return app.test_client()


@pytest.fixture(autouse=True)
def reset_store():
    from src.routes import api as api_module

    api_module.simulation_store.clear()
    yield
    api_module.simulation_store.clear()


# --- Simulation store unit tests ---


def test_simulation_store_records_and_lists():
    store = SimulationStore(max_size=3)
    eid = store.add(
        DEFAULT_VALUES,
        {"I1": 1.0, "I2": 2.0, "I3": 3.0},
        {"I1": "x", "I2": "y", "I3": "z"},
        10.0,
    )
    items = store.list()
    assert len(items) == 1
    assert items[0]["id"] == eid
    assert items[0]["currents"]["I2"] == 2.0


def test_simulation_store_respects_max_size():
    store = SimulationStore(max_size=2)
    store.add(DEFAULT_VALUES, {"I1": 0, "I2": 0, "I3": 0}, {}, 0)
    store.add(DEFAULT_VALUES, {"I1": 0, "I2": 0, "I3": 0}, {}, 0)
    store.add(DEFAULT_VALUES, {"I1": 0, "I2": 0, "I3": 0}, {}, 0)
    assert store.size() == 2


def test_simulation_store_aggregate_report_counts_severities():
    store = SimulationStore()
    store.add(DEFAULT_VALUES, {"I1": 0.5, "I2": 5.0, "I3": 25.0}, {}, 1.0)
    store.add(DEFAULT_VALUES, {"I1": 100.0, "I2": 0.0, "I3": 0.0}, {}, 1.0)
    report = store.aggregate_report()
    assert report["total_simulations"] == 2
    assert report["by_severity"]["baja"] == 3
    assert report["by_severity"]["normal"] == 1
    assert report["by_severity"]["alta"] == 1
    assert report["by_severity"]["critica"] == 1


def test_simulation_store_clear_removes_all():
    store = SimulationStore()
    store.add(DEFAULT_VALUES, {"I1": 0, "I2": 0, "I3": 0}, {}, 0)
    removed = store.clear()
    assert removed == 1
    assert store.size() == 0


# --- CSV/PDF exporter unit tests ---


def test_build_csv_contains_headers_and_values():
    simulation = {
        "id": "abc123",
        "timestamp": "2026-06-29T00:00:00Z",
        "params": DEFAULT_VALUES,
        "currents": {"I1": 1.0, "I2": 2.0, "I3": 3.0},
        "interpretations": {"I1": "x", "I2": "y", "I3": "z"},
        "power_total": 42.0,
    }
    csv_bytes = build_csv(simulation)
    text = csv_bytes.decode("utf-8")
    reader = csv.reader(io.StringIO(text))
    rows = list(reader)
    assert any("Malla" in cell for row in rows for cell in row)
    assert any("R1" == cell for row in rows for cell in row)
    assert any("I1" == cell for row in rows for cell in row)


def test_build_pdf_returns_valid_signature():
    simulation = {
        "id": "abc123",
        "timestamp": "2026-06-29T00:00:00Z",
        "params": DEFAULT_VALUES,
        "currents": {"I1": 1.0, "I2": 2.0, "I3": 3.0},
        "interpretations": {"I1": "x", "I2": "y", "I3": "z"},
        "power_total": 42.0,
    }
    pdf_bytes = build_pdf(simulation)
    assert pdf_bytes[:4] == b"%PDF"


# --- API integration tests ---


def test_api_calculate_records_simulation_in_history(client):
    response = client.post("/api/calculate", json=DEFAULT_VALUES)
    assert response.status_code == 200
    payload = response.get_json()
    assert payload["success"] is True
    assert "id" in payload

    history = client.get("/api/history").get_json()
    assert history["count"] >= 1
    assert any(item["id"] == payload["id"] for item in history["items"])


def test_api_history_clear_empties_list(client):
    client.post("/api/calculate", json=DEFAULT_VALUES)
    response = client.delete("/api/history")
    assert response.status_code == 200
    body = response.get_json()
    assert body["success"] is True
    assert body["removed"] >= 1
    assert client.get("/api/history").get_json()["count"] == 0


def test_api_history_get_returns_404_for_unknown_id(client):
    response = client.get("/api/history/no-such-id")
    assert response.status_code == 404
    assert response.get_json()["error"]["code"] == "NOT_FOUND"


def test_api_export_csv_from_payload(client):
    response = client.post(
        "/api/export/csv",
        data=json.dumps(DEFAULT_VALUES),
        content_type="application/json",
    )
    assert response.status_code == 200
    assert "text/csv" in response.headers["Content-Type"]
    assert b"Malla" in response.data


def test_api_export_pdf_from_payload(client):
    response = client.post(
        "/api/export/pdf",
        data=json.dumps(DEFAULT_VALUES),
        content_type="application/json",
    )
    assert response.status_code == 200
    assert response.headers["Content-Type"] == "application/pdf"
    assert response.data[:4] == b"%PDF"


def test_api_export_csv_uses_stored_id(client):
    calc = client.post("/api/calculate", json=DEFAULT_VALUES).get_json()
    response = client.post(
        "/api/export/csv",
        data=json.dumps({"id": calc["id"]}),
        content_type="application/json",
    )
    assert response.status_code == 200
    assert calc["id"] in response.headers["Content-Disposition"]


def test_api_export_returns_415_when_not_json(client):
    response = client.post("/api/export/csv", data="not json", content_type="text/plain")
    assert response.status_code == 415


def test_api_export_returns_400_when_json_malformed(client):
    response = client.post("/api/export/pdf", data="{", content_type="application/json")
    assert response.status_code == 400


def test_api_export_returns_400_when_validation_fails(client):
    bad = {**DEFAULT_VALUES, "R1": -1}
    response = client.post("/api/export/csv", data=json.dumps(bad), content_type="application/json")
    assert response.status_code == 400


def test_api_compare_returns_diffs(client):
    baseline = {
        "R1": 2,
        "R2": 4,
        "R3": 3,
        "R4": 6,
        "R5": 5,
        "R6": 2,
        "V1": 12,
        "V2": 0,
        "V3": 0,
    }
    optimized = {
        "R1": 1.5,
        "R2": 3.5,
        "R3": 2.5,
        "R4": 5,
        "R5": 4.5,
        "R6": 1.8,
        "V1": 12,
        "V2": 0,
        "V3": 0,
    }
    response = client.post(
        "/api/compare",
        data=json.dumps({"baseline": baseline, "optimized": optimized}),
        content_type="application/json",
    )
    assert response.status_code == 200
    body = response.get_json()
    assert body["success"] is True
    assert body["comparison"]["deltas"]["power_total"]["absolute"] != 0
    assert body["comparison"]["power_reduction_label"] in {"mejora", "empeoramiento", "sin cambio"}


def test_api_compare_returns_400_when_payload_invalid(client):
    response = client.post(
        "/api/compare",
        data=json.dumps({"baseline": DEFAULT_VALUES}),
        content_type="application/json",
    )
    assert response.status_code == 400


def test_api_compare_returns_415_when_not_json(client):
    response = client.post("/api/compare", data="x", content_type="text/plain")
    assert response.status_code == 415


def test_api_report_returns_aggregate_zero_when_empty(client):
    response = client.get("/api/report")
    assert response.status_code == 200
    body = response.get_json()
    assert body["report"]["total_simulations"] == 0
    assert body["history_capacity"] > 0


def test_api_report_reflects_recent_simulations(client):
    client.post("/api/calculate", json=DEFAULT_VALUES)
    client.post("/api/calculate", json=DEFAULT_VALUES)
    body = client.get("/api/report").get_json()
    assert body["report"]["total_simulations"] == 2
    assert body["report"]["max_power_watts"] > 0


def test_api_metrics_includes_history_size(client):
    client.post("/api/calculate", json=DEFAULT_VALUES)
    body = client.get("/api/metrics").get_json()
    assert body["history_size"] == 1
    assert body["history_capacity"] > 0


# --- Phase 6: UI features ---


def test_web_index_renders_theme_toggle_and_toast_container(client):
    response = client.get("/")
    body = response.get_data(as_text=True)
    assert response.status_code == 200
    assert 'id="theme-toggle"' in body
    assert 'id="toast-container"' in body
    assert "icon-moon" in body
    assert "icon-sun" in body


def test_web_index_renders_advanced_section_with_icons(client):
    body = client.get("/").get_data(as_text=True)
    assert "advanced-section" in body
    assert "subsection-icon" in body
    assert "Historial de simulaciones" in body
    assert "Comparar escenarios" in body
    assert "Reporte técnico agregado" in body


def test_web_index_renders_result_cards_with_data_severity(client):
    response = client.post(
        "/",
        data={
            "R1": "2",
            "R2": "4",
            "R3": "3",
            "R4": "6",
            "R5": "5",
            "R6": "2",
            "V1": "12",
            "V2": "0",
            "V3": "0",
            "resistance_count": "6",
            "voltage_count": "3",
        },
        follow_redirects=True,
    )
    body = response.get_data(as_text=True)
    assert response.status_code == 200
    assert 'data-severity="normal"' in body
    assert "results-actions-sticky" in body
    assert 'id="btn-export-csv"' in body
    assert 'id="btn-export-pdf"' in body


def test_web_index_renders_data_severity_critical_for_high_current(client):
    response = client.post(
        "/",
        data={
            "R1": "0.01",
            "R2": "0.01",
            "R3": "0.01",
            "R4": "0.01",
            "R5": "0.01",
            "R6": "0.01",
            "V1": "500",
            "V2": "500",
            "V3": "500",
            "resistance_count": "6",
            "voltage_count": "3",
        },
        follow_redirects=True,
    )
    body = response.get_data(as_text=True)
    assert response.status_code == 200
    assert 'data-severity="critica"' in body
