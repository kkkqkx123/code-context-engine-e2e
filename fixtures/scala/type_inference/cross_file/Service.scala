package app.service

import app.models.{User, UserFactory}

object Service {
  def renderGreeting(user: User): String = user.greet()

  def main(args: Array[String]): Unit = {
    val user = UserFactory.createUser("Alice")
    println(renderGreeting(user))
    println(user.greet())
  }
}
