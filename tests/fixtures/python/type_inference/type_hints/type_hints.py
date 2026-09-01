from typing import List, Dict, Optional, Tuple, Any


def process_items(items: List[str]) -> Dict[str, int]:
    result: Dict[str, int] = {}
    for item in items:
        result[item] = len(item)
    return result


def safe_divide(a: float, b: float) -> Optional[float]:
    if b == 0.0:
        return None
    return a / b


def get_user_info(user_id: int) -> Tuple[str, int, str]:
    name = f"user_{user_id}"
    age = user_id * 10
    email = f"{name}@example.com"
    return (name, age, email)


def identity(x: Any) -> Any:
    return x


def wrap_value(value: str) -> List[str]:
    return [value]


class Container:
    def __init__(self, value: str) -> None:
        self.value = value

    def get_value(self) -> str:
        return self.value

    def duplicate(self) -> "Container":
        return Container(self.value)


def process_pair(a: int, b: str) -> str:
    return f"{a}: {b}"


def main() -> None:
    items = ["hello", "world"]
    result = process_items(items)
    divided = safe_divide(10.0, 3.0)
    user = get_user_info(1)
    container = Container("test")
    dup = container.duplicate()
    wrapped = wrap_value("item")
    pair_result = process_pair(42, "answer")
