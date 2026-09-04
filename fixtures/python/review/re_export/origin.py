"""Origin module defining shared symbols."""


def format_name(name: str) -> str:
    return f"hello {name}"


class Greeter:
    def greet(self, name: str) -> str:
        return format_name(name)
