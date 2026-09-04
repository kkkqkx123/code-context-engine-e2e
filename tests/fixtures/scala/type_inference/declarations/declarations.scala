case class Container[T](value: T) {
  def duplicate(): Container[T] = Container(value)
}

case class Pair[A, B](first: A, second: B) {
  def swap(): Pair[B, A] = Pair(second, first)
}

object GenericsDemo {
  def identity[T](x: T): T = x

  def wrapInList[T](item: T): List[T] = List(item)

  def toMap[K, V](key: K, value: V): Map[K, V] = Map(key -> value)

  def maxOf[T <: Comparable[T]](a: T, b: T): T =
    if (a.compareTo(b) >= 0) a else b

  def main(args: Array[String]): Unit = {
    val id: String = identity("test")
    val list = wrapInList("item")
    val map = toMap("key", 100)
    val container = Container("value")
    val dup = container.duplicate()
    val p = Pair(1, "one")
    val swapped = p.swap()
    val biggest: Int = maxOf(10, 20)
    println(s"$id $list $map $dup $swapped $biggest")
  }
}
