case class User(name: String, age: Int)

object ForComprehension {
  def usernames(users: List[User]): List[String] = {
    for {
      user <- users
      if user.age >= 18
    } yield user.name
  }

  def pairs(): List[(Int, String)] = {
    for {
      n <- List(1, 2, 3)
      label <- List("a", "b")
    } yield (n, label)
  }

  def main(args: Array[String]): Unit = {
    val users = List(User("ada", 36), User("bob", 12))
    println(usernames(users))
    println(pairs())
  }
}
