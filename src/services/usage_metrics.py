from collections import defaultdict
from threading import Lock
from typing import Dict


class UsageMetrics:
    def __init__(self) -> None:
        self._lock = Lock()
        self._by_route: Dict[str, int] = defaultdict(int)
        self._total_requests = 0

    def record_request(self, method: str, path: str) -> None:
        key = f"{method} {path}"
        with self._lock:
            self._total_requests += 1
            self._by_route[key] += 1

    def snapshot(self) -> Dict[str, object]:
        with self._lock:
            return {
                "total_requests": self._total_requests,
                "requests_by_route": dict(self._by_route),
            }


usage_metrics = UsageMetrics()
