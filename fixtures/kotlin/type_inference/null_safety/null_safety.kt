fun lengthOrDefault(value: String?): Int {
    return value?.length ?: 0
}

fun shout(value: String?): String {
    if (value != null) {
        return value.uppercase()
    }
    return "empty"
}

fun forced(value: String?): Int {
    return value!!.length
}

fun elvisChain(first: String?, second: String?): String {
    return first ?: second ?: "none"
}

fun main() {
    println(lengthOrDefault("hello"))
    println(shout(null))
    println(forced("ada"))
    println(elvisChain(null, "fallback"))
}
