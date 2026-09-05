fun handleNegated(value: Any): String {
    if (value !is String) {
        return "not a string"
    }
    return value.uppercase()
}

fun classifyWhen(value: Any): String {
    return when {
        value is String -> value.uppercase()
        value is Int -> "number: $value"
        else -> "other"
    }
}

fun main() {
    println(handleNegated(42))
    println(classifyWhen("hello"))
}
