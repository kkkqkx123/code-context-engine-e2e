from typing import Optional, Union


def handle_optional(value: Optional[str]) -> str:
    if value is not None:
        return value
    return "default"


def handle_isinstance(value: Union[str, int]) -> str:
    if isinstance(value, str):
        return value.upper()
    return str(value)


def narrow_with_match(value: Union[str, int, None]) -> str:
    match value:
        case str(s):
            return s
        case int(n):
            return str(n)
        case None:
            return "none"
    return "unknown"


def handle_truthiness(value: Optional[str]) -> str:
    if value:
        return value
    return "empty or None"


def main() -> None:
    opt: Optional[str] = "hello"
    print(handle_optional(opt))
    print(handle_isinstance("test"))
    print(narrow_with_match(42))
    print(handle_truthiness("present"))
