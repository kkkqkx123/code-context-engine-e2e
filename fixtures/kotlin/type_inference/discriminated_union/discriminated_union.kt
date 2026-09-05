sealed class Result {
    data class Success(val value: String) : Result()
    data class Failure(val message: String) : Result()
    object Pending : Result()
}

fun describe(result: Result): String {
    return when (result) {
        is Result.Success -> result.value
        is Result.Failure -> result.message
        is Result.Pending -> "pending"
    }
}

fun handleIs(result: Result): String {
    if (result is Result.Success) {
        return result.value
    }
    return "not success"
}

fun main() {
    println(describe(Result.Success("done")))
    println(handleIs(Result.Failure("oops")))
}
