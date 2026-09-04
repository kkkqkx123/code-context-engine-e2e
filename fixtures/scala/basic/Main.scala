object Main {
  def main(args: Array[String]): Unit = {
    val sum = Calculator.add(1, 2)
    val product = Calculator.multiply(sum, 3)
    val summary = Calculator.summarize(1, 2)
    val greeter: Greeter = new FriendlyGreeter()
    println(s"$product $summary ${greeter.greet("World")}")
  }
}
