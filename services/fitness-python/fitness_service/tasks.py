from __future__ import annotations

from typing import List, Tuple


def get_cases(task: str) -> List[Tuple[float, float]]:
    xs = list(range(-5, 6))
    if task == "poly2" or not task:
        return [(float(x), poly2(float(x))) for x in xs]
    # Default to poly2 for unknown tasks to keep behavior deterministic.
    return [(float(x), poly2(float(x))) for x in xs]


def poly2(x: float) -> float:
    return x * x + 3.0 * x + 2.0
