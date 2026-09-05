package app.models

case class User(name: String, age: Int) {
  def greet(): String = s"Hello, $name!"
}

object UserFactory {
  def createUser(name: String): User = User(name, 30)
}
