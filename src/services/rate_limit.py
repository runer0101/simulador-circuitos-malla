"""Throttling for the expensive POST endpoints.

The counter lives in the process, so with N Gunicorn workers the effective ceiling is N times the
configured one. A shared counter would need Redis; this service is meant to run as one small
container, so the per-process limit is deliberate and documented in the README.
"""

import os
import threading
import time
from collections import deque
from typing import Callable

# A client that never repeats an address still leaves a key behind; prune before the map grows
# without bound. O(n) once per PRUNE_AT new keys is cheap next to a matrix solve.
PRUNE_AT = 4096


class SlidingWindowLimiter:
    """Count events per key inside a moving window."""

    def __init__(self, window_seconds: float, clock: Callable[[], float] = time.monotonic) -> None:
        self.window = window_seconds
        self._clock = clock
        self._hits: dict[str, deque[float]] = {}
        self._lock = threading.Lock()

    def check(self, key: str, limit: int) -> tuple[bool, int]:
        """Register one hit for key. Return (allowed, seconds_until_a_slot_frees)."""
        now = self._clock()
        with self._lock:
            hits = self._hits.setdefault(key, deque())
            while hits and now - hits[0] >= self.window:
                hits.popleft()
            if len(hits) >= limit:
                return False, max(1, int(self.window - (now - hits[0])) + 1)
            hits.append(now)
            if len(self._hits) > PRUNE_AT:
                self._prune(now)
            return True, 0

    def _prune(self, now: float) -> None:
        stale = [key for key, hits in self._hits.items() if not hits or now - hits[-1] >= self.window]
        for key in stale:
            del self._hits[key]

    def tracked_keys(self) -> int:
        with self._lock:
            return len(self._hits)

    def reset(self) -> None:
        with self._lock:
            self._hits.clear()


api_rate_limiter = SlidingWindowLimiter(float(os.environ.get("RATE_LIMIT_WINDOW_SECONDS", "60")))
