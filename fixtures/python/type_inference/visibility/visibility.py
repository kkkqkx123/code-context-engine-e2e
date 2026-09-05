"""Module demonstrating Python visibility conventions."""

__all__ = ['public_func', 'PublicClass']


def public_func():
    """Public function (in __all__)."""
    return "public"


def _private_func():
    """Private function (underscore prefix)."""
    return "private"


def __dunder_func__():
    """Dunder function (special method)."""
    return "dunder"


class PublicClass:
    """Public class (in __all__)."""
    
    def public_method(self):
        """Public method."""
        return "public"
    
    def _private_method(self):
        """Private method."""
        return "private"


class _PrivateClass:
    """Private class (underscore prefix)."""
    
    def method(self):
        return "private class"
