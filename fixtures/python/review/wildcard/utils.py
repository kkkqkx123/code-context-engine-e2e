"""Utility module with wildcard-visible symbols."""


def alpha() -> str:
    return "alpha"


def beta() -> str:
    return "beta"


def _private_helper() -> str:
    return "helper"


__all__ = ["alpha", "beta"]
