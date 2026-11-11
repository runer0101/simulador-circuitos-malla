import pytest

from src.services.rate_limit import api_rate_limiter
from src.services.simulation_store import simulation_store


@pytest.fixture(autouse=True)
def _reset_shared_state():
    """The limiter and the history are process-wide; a leftover entry makes assertions flaky."""
    api_rate_limiter.reset()
    simulation_store.clear()
    yield
    api_rate_limiter.reset()
    simulation_store.clear()
