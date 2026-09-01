package app

import kotlin.math.sqrt

fun add(a: Int, b: Int): Int = a + b

fun isPrime(n: Int): Boolean {
    if (n < 2) return false
    for (i in 2..sqrt(n.toDouble()).toInt()) {
        if (n % i == 0) return false
    }
    return true
}

class Greeter(private val name: String) {
    fun greet(): String = "Hello, $name!"
}

fun main() {
    val sum = add(1, 2)
    println("Sum: $sum")
    println("Is 7 prime? ${isPrime(7)}")
    val greeter = Greeter("World")
    println(greeter.greet())
}
