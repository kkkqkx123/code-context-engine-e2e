object Demo {
  def run(): Unit = {
    val alice = User("Alice", 30)
    val root = Admin("root", 9)
    Printer.printAll(List(alice, root))
    println(Printer.greet("Bob"))
  }

  def main(args: Array[String]): Unit = run()
}
