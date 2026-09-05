public sealed interface Shape permits Circle, Rectangle {

    record Circle(double radius) implements Shape {}

    record Rectangle(double width, double height) implements Shape {}

    static double area(Shape shape) {
        if (shape instanceof Circle c) {
            return 3.14 * c.radius() * c.radius();
        } else if (shape instanceof Rectangle r) {
            return r.width() * r.height();
        }
        return 0.0;
    }

    static String describe(Shape shape) {
        if (shape instanceof Circle) {
            return "round";
        }
        return "angular";
    }
}
