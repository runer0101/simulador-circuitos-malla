"""Servicio de historial de simulaciones en memoria.

Mantiene un buffer circular de las últimas N simulaciones realizadas
para permitir su revisión, exportación y análisis agregado.
"""

from __future__ import annotations

import threading
import uuid
from collections import deque
from datetime import datetime, timezone
from typing import Any, Deque, Dict, List, Optional


class SimulationStore:
    def __init__(self, max_size: int = 50) -> None:
        self._max_size = max_size
        self._items: Deque[Dict[str, Any]] = deque(maxlen=max_size)
        self._lock = threading.Lock()

    def add(
        self,
        params: Dict[str, float],
        currents: Dict[str, float],
        interpretations: Dict[str, str],
        power_total: float,
    ) -> str:
        entry_id = uuid.uuid4().hex[:12]
        entry: Dict[str, Any] = {
            "id": entry_id,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "params": {k: float(v) for k, v in params.items()},
            "currents": {k: float(v) for k, v in currents.items()},
            "interpretations": dict(interpretations),
            "power_total": float(power_total),
        }
        with self._lock:
            self._items.appendleft(entry)
        return entry_id

    def list(self) -> List[Dict[str, Any]]:
        with self._lock:
            return [dict(item) for item in self._items]

    def get(self, entry_id: str) -> Optional[Dict[str, Any]]:
        with self._lock:
            for item in self._items:
                if item["id"] == entry_id:
                    return dict(item)
        return None

    def clear(self) -> int:
        with self._lock:
            count = len(self._items)
            self._items.clear()
            return count

    def size(self) -> int:
        with self._lock:
            return len(self._items)

    def capacity(self) -> int:
        return self._max_size

    def aggregate_report(self) -> Dict[str, Any]:
        with self._lock:
            items = list(self._items)
        if not items:
            return {
                "total_simulations": 0,
                "by_severity": {"baja": 0, "normal": 0, "alta": 0, "critica": 0},
                "max_current_observed": 0.0,
                "avg_power_watts": 0.0,
                "min_power_watts": 0.0,
                "max_power_watts": 0.0,
                "first_simulation_at": None,
                "last_simulation_at": None,
            }

        def severity(c: float) -> str:
            m = abs(c)
            if m < 1:
                return "baja"
            if m < 10:
                return "normal"
            if m < 50:
                return "alta"
            return "critica"

        by_severity: Dict[str, int] = {"baja": 0, "normal": 0, "alta": 0, "critica": 0}
        max_current = 0.0
        powers: List[float] = []
        for item in items:
            for current in item["currents"].values():
                max_current = max(max_current, abs(current))
                by_severity[severity(current)] += 1
            powers.append(item["power_total"])

        return {
            "total_simulations": len(items),
            "by_severity": by_severity,
            "max_current_observed": float(max_current),
            "avg_power_watts": float(sum(powers) / len(powers)),
            "min_power_watts": float(min(powers)),
            "max_power_watts": float(max(powers)),
            "first_simulation_at": items[-1]["timestamp"],
            "last_simulation_at": items[0]["timestamp"],
        }


simulation_store = SimulationStore()
