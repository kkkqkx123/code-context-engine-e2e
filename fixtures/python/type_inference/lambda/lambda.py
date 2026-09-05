"""Lambda type inference fixture.

Covers an annotated lambda binding (inferable from the annotation),
a higher-order call passing a lambda, and an unannotated lambda
where the inferer stays conservative.
"""

from typing import Callable

formatter: Callable[[int], str] = lambda x: str(x)


def apply_twice(func: Callable[[int], int], value: int) -> int:
    return func(func(value))


def main() -> None:
    doubled = apply_twice(lambda x: x * 2, 21)
    print(formatter(doubled))
