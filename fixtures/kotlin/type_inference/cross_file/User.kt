package app.models

data class User(val name: String, val age: Int) {
    fun greet(): String = "Hello, $name!"
}

fun loadUser(name: String): User = User(name, 30)
