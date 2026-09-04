fun <T> identity(x: T): T = x

fun <T> wrapInList(item: T): List<T> = listOf(item)

fun <K, V> toMap(key: K, value: V): Map<K, V> = mapOf(key to value)

class Container<T>(val value: T) {
    fun duplicate(): Container<T> = Container(value)
}

class Pair<A, B>(val first: A, val second: B) {
    fun swap(): Pair<B, A> = Pair(second, first)
}

fun <T : Comparable<T>> maxOf(a: T, b: T): T = if (a >= b) a else b

fun main() {
    val id: String = identity("test")
    val list = wrapInList("item")
    val map = toMap("key", 100)
    val container = Container("value")
    val dup = container.duplicate()
    val p = Pair(1, "one")
    val swapped = p.swap()
    val biggest: Int = maxOf(10, 20)
    println("$id $list $map $dup $swapped $biggest")
}
