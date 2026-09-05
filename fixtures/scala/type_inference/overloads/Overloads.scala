package app

object Overloads {
  def combine(a: Int, b: Int): Int = a + b

  def combine(a: String, b: String): String = a + b

  def combine(a: Int, b: String): String = s"$a$b"

  def run(): String = {
    val ints = combine(1, 2)
    val strs = combine("a", "b")
    val mixed = combine(1, "b")
    s"$ints$strs$mixed"
  }
}
