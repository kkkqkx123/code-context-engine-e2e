trait Printable {
  def render(): String
}

case class User(name: String, age: Int) extends Printable {
  override def render(): String = s"$name is $age"
}

case class Admin(name: String, level: Int) extends Printable {
  override def render(): String = s"admin $name ($level)"
}

object Printer {
  def printAll(items: List[Printable]): Unit =
    items.foreach(i => println(i.render()))

  implicit def stringToUser(name: String): User = User(name, 0)

  def greet(user: User): String = s"hello ${user.name}"
}
