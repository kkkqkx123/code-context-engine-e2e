"""Consumer testing visibility rules."""

from visibility import public_func, PublicClass


def consume():
    """Consume public symbols."""
    result = public_func()
    obj = PublicClass()
    method_result = obj.public_method()
    return result, method_result
