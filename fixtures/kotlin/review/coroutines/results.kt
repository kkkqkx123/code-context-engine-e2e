package app

sealed class Result {
  data class Success(val value: String) : Result()
  data class Failure(val message: String) : Result()
  data object Loading : Result()
}

fun String.shout(): String = this.uppercase() + "!"

suspend fun fetchResult(id: Int): Result {
  return Result.Success("item-$id")
}

fun describe(result: Result): String {
  return when (result) {
    is Result.Success -> result.value.shout()
    is Result.Failure -> "error: ${result.message}"
    Result.Loading -> "loading"
  }
}
