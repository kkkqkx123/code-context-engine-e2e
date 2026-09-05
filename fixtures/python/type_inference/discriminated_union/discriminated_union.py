"""Discriminated union narrowing via tuple isinstance and kind field."""

from typing import Union


class Circle:
    kind = "circle"

    def __init__(self, radius: float) -> None:
        self.radius = radius


class Rectangle:
    kind = "rectangle"

    def __init__(self, width: float, height: float) -> None:
        self.width = width
        self.height = height


def area(shape: Union[Circle, Rectangle]) -> float:
    if isinstance(shape, Circle):
        return 3.14 * shape.radius * shape.radius
    return shape.width * shape.height


def describe(shape: Union[Circle, Rectangle]) -> str:
    if isinstance(shape, (Circle, Rectangle)):
        if shape.kind == "circle":
            return "round"
        return "angular"
    return "unknown"


def main() -> None:
    print(area(Circle(1.0)))
    print(describe(Rectangle(2.0, 3.0)))
