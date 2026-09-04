"""Service layer exercising cross-file constructor + method calls."""

from models import User, load_user


def render_greeting(user: User) -> str:
    return user.greet()


def main() -> None:
    user = load_user("Alice")
    print(render_greeting(user))
    print(user.greet())
