fun parseNumber(text: String): Int? {
    return text.toIntOrNull()
}

fun main() {
    val greeting = "hello".let { it.uppercase() }
    val alsoValue = "ada".also { println(it) }
    val computed = parseNumber("42")?.let { it * 2 } ?: 0
    val applied = StringBuilder().apply { append("a"); append("b") }.toString()
    println("$greeting $alsoValue $computed $applied")
}
