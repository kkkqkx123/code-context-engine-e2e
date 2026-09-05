package app.service

import app.models.User
import app.models.loadUser

fun renderGreeting(user: User): String = user.greet()

fun main() {
    val user = loadUser("Alice")
    println(renderGreeting(user))
    println(user.greet())
}
