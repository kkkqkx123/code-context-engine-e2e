trait Greeter {
  def greet(name: String): String
}

class FriendlyGreeter extends Greeter {
  override def greet(name: String): String = s"Hello, $name!"
}

case class CalculatorResult(sum: Int, product: Int)

object Calculator {
  def add(a: Int, b: Int): Int = a + b

  def multiply(a: Int, b: Int): Int = a * b

  def summarize(a: Int, b: Int): CalculatorResult =
    CalculatorResult(add(a, b), multiply(a, b))
}
