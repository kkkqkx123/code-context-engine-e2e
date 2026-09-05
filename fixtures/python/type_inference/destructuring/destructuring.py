"""Tuple, dict, and match-case destructuring."""


def split_pair(pair: tuple[int, str]) -> str:
    first, second = pair
    return f"{first}: {second}"


def lookup_user(payload: dict[str, str]) -> str:
    name = payload["name"]
    return name


def handle_point(value: tuple[int, int]) -> str:
    match value:
        case (0, 0):
            return "origin"
        case (x, 0):
            return f"x={x}"
        case (x, y):
            return f"{x},{y}"


def main() -> None:
    print(split_pair((1, "one")))
    print(lookup_user({"name": "ada"}))
    print(handle_point((3, 4)))
