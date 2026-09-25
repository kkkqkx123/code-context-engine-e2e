from dataclasses import dataclass
from typing import Iterable


@dataclass
class InvoiceItem:
    name: str
    price: float
    quantity: int

    def total(self) -> float:
        return self.price * self.quantity


class Invoice:
    def __init__(self, customer: str) -> None:
        self.customer = customer
        self.items: list[InvoiceItem] = []

    def add_item(self, item: InvoiceItem) -> None:
        self.items.append(item)

    def subtotal(self) -> float:
        return sum(item.total() for item in self.items)

    def tax(self, rate: float = 0.08) -> float:
        return round(self.subtotal() * rate, 2)

    def render(self) -> str:
        lines = [f"Invoice for {self.customer}"]
        lines.extend(f"- {item.name}: {item.total():.2f}" for item in self.items)
        lines.append(f"Tax: {self.tax():.2f}")
        return "\n".join(lines)


def build_invoice(customer: str, items: Iterable[InvoiceItem]) -> Invoice:
    invoice = Invoice(customer)
    for item in items:
        invoice.add_item(item)
    return invoice
