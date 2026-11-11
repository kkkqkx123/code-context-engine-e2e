from . import sibling
from .models import Base
from os import path
import json
import pkg.loader as loader
from pkg import tool, other as renamed
from pkg.sub import (alpha, beta)
from ..shared import common
from .. import top
from pkg import *

if TYPE_CHECKING:
    from typing import Optional

try:
    from fallback import impl
except ImportError:
    impl = None


class Handler(Base):
    def run(self, items):
        super().run(items)
        self.prepare()
        json.dumps({})
        return sorted(items)
