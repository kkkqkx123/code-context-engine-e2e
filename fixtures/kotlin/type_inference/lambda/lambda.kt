val toLabel: (Int) -> String = { n -> "n=$n" }

fun applyTwice(value: Int, fn: (Int) -> Int): Int = fn(fn(value))

fun main() {
    val doubled = applyTwice(21) { x -> x * 2 }
    val lengths = listOf("a", "bb", "ccc").map { it.length }
    println("$toLabel $doubled $lengths")
}
