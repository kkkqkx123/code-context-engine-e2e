sealed trait Shape
case class Circle(radius: Double) extends Shape
case class Rectangle(width: Double, height: Double) extends Shape
case object Unknown extends Shape

object ControlFlowDemo {
  def describe(x: Any): String = {
    if (x.isInstanceOf[String]) {
      x.asInstanceOf[String].toUpperCase
    } else if (x.isInstanceOf[Int]) {
      s"number: $x"
    } else {
      "unknown"
    }
  }

  def matchShape(shape: Shape): String = shape match {
    case c: Circle => s"circle r=${c.radius}"
    case r: Rectangle => s"rect ${r.width}x${r.height}"
    case _ => "unknown"
  }

  def matchValue(x: Any): String = x match {
    case s: String => s.toUpperCase
    case n: Int => n.toString
    case _ => "other"
  }

  def main(args: Array[String]): Unit = {
    println(describe("hello"))
    println(describe(42))
    println(matchShape(Circle(1.0)))
    println(matchValue("test"))
  }
}
