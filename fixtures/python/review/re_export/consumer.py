"""Consumer importing through the re-export chain."""

from middle import format_name, Greeter


def render(name: str) -> str:
    greeter = Greeter()
    return greeter.greet(name) + "|" + format_name(name)
