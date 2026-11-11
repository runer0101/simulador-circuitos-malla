import pytest

from src.services.rate_limit import api_rate_limiter


@pytest.fixture(autouse=True)
def _reset_rate_limit():
    """The limiter is process-wide; without a reset the suite's own traffic trips the limit."""
    api_rate_limiter.reset()
    yield
    api_rate_limiter.reset()
