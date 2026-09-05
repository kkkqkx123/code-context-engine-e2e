package app

class Overloads {
    fun combine(a: Int, b: Int): Int = a + b

    fun combine(a: String, b: String): String = a + b

    fun combine(a: Int, b: String): String = "$a$b"

    fun run(): String {
        val ints = combine(1, 2)
        val strs = combine("a", "b")
        val mixed = combine(1, "b")
        return "$ints$strs$mixed"
    }
}
