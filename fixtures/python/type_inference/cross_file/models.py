"""Shared domain models imported across files."""


class User:
    def __init__(self, name: str, age: int) -> None:
        self.name = name
        self.age = age

    def greet(self) -> str:
        return f"Hello, {self.name}!"


def load_user(name: str) -> User:
    return User(name, 30)
