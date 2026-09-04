package app

suspend fun loadAndDescribe(id: Int): String {
  val result = fetchResult(id)
  return describe(result)
}

fun main() {
  println("coroutines demo".shout())
  println(describe(Result.Success("hi")))
  println(describe(Result.Loading))
}
