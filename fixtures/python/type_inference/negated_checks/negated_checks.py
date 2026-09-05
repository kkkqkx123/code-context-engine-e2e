"""Negated isinstance / None guards."""

from typing import Optional, Union


def handle_not_str(value: Union[str, int]) -> str:
    if not isinstance(value, str):
        return f"number {value}"
    return value.upper()


def handle_is_not_none(value: Optional[str]) -> str:
    if value is not None:
        return value
    return "missing"


def handle_not_truthy(value: Optional[str]) -> str:
    if not value:
        return "empty"
    return value


def main() -> None:
    print(handle_not_str(42))
    print(handle_is_not_none("hello"))
    print(handle_not_truthy(""))
