sealed class Result {
    data class Success(val value: String) : Result()
    data class Error(val message: String) : Result()
    object Pending : Result()
}

fun handleIs(value: Any): String {
    if (value is String) {
        return value.uppercase()
    }
    if (value is Int) {
        return "number: $value"
    }
    return "unknown"
}

fun handleWhen(value: Any): String {
    return when (value) {
        is String -> value.uppercase()
        is Int -> "number: $value"
        else -> "unknown"
    }
}

fun handleResult(result: Result): String {
    return when (result) {
        is Result.Success -> result.value
        is Result.Error -> result.message
        is Result.Pending -> "pending"
    }
}

fun handleKind(kind: String, payload: Any): String {
    if (kind == "text") {
        return payload.toString()
    }
    return "other"
}

fun main() {
    println(handleIs("hello"))
    println(handleIs(42))
    println(handleWhen("test"))
    println(handleResult(Result.Success("done")))
    println(handleKind("text", "payload"))
}
