from src.app_factory import create_app
from src.config import EXAMPLE_VALUES
from src.services.rate_limit import SlidingWindowLimiter, api_rate_limiter


def _client():
    api_rate_limiter.reset()
    app = create_app()
    app.config["TESTING"] = True
    return app.test_client(), app


def test_window_releases_capacity_after_it_passes():
    now = [100.0]
    limiter = SlidingWindowLimiter(60, clock=lambda: now[0])

    assert limiter.check("cliente", 2) == (True, 0)
    assert limiter.check("cliente", 2) == (True, 0)

    allowed, wait = limiter.check("cliente", 2)
    assert allowed is False
    assert wait == 61

    now[0] += 61
    assert limiter.check("cliente", 2) == (True, 0)


def test_keys_do_not_share_the_quota():
    limiter = SlidingWindowLimiter(60)

    assert limiter.check("uno", 1) == (True, 0)
    assert limiter.check("dos", 1) == (True, 0)
    assert limiter.check("uno", 1)[0] is False


def test_calculate_is_throttled_with_a_retry_after_header():
    client, app = _client()
    app.config["RATE_LIMIT_REQUESTS"] = 2

    assert client.post("/api/calculate", json=EXAMPLE_VALUES).status_code == 200
    assert client.post("/api/calculate", json=EXAMPLE_VALUES).status_code == 200

    response = client.post("/api/calculate", json=EXAMPLE_VALUES)

    assert response.status_code == 429
    assert int(response.headers["Retry-After"]) > 0
    body = response.get_json()
    assert body["success"] is False
    assert body["error"]["code"] == "RATE_LIMITED"
    assert body["error"]["message"]


def test_get_requests_are_not_throttled():
    client, app = _client()
    app.config["RATE_LIMIT_REQUESTS"] = 1

    client.post("/api/calculate", json=EXAMPLE_VALUES)

    for _ in range(5):
        assert client.get("/api/health").status_code == 200


def test_html_form_post_is_throttled_too():
    client, app = _client()
    app.config["RATE_LIMIT_REQUESTS"] = 1
    form = "R1=2&R2=4&R3=3&R4=6&R5=5&R6=2&V1=12&V2=0&V3=0"

    assert client.post("/", data=form).status_code == 200
    assert client.post("/", data=form).status_code == 429


def test_stale_keys_are_pruned():
    now = [0.0]
    limiter = SlidingWindowLimiter(60, clock=lambda: now[0])
    for index in range(5000):
        limiter.check(f"cliente-{index}", 1)

    now[0] += 61
    limiter.check("cliente-nuevo", 1)

    assert limiter.tracked_keys() < 100


def test_throttled_request_does_not_touch_the_history():
    client, app = _client()
    app.config["RATE_LIMIT_REQUESTS"] = 1
    client.post("/api/calculate", json=EXAMPLE_VALUES)
    before = client.get("/api/history").get_json()["count"]

    assert client.post("/api/calculate", json=EXAMPLE_VALUES).status_code == 429

    assert client.get("/api/history").get_json()["count"] == before
