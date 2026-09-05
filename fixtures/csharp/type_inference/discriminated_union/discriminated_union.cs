namespace DiscriminatedApp
{
    public abstract class Shape
    {
        public abstract string Kind { get; }
    }

    public class Circle : Shape
    {
        public override string Kind => "Circle";
        public double Radius { get; set; }
    }

    public class Rectangle : Shape
    {
        public override string Kind => "Rectangle";
        public double Width { get; set; }
        public double Height { get; set; }
    }

    public class ShapeService
    {
        public static string Describe(Shape shape)
        {
            if (shape.Kind == "Circle") {
                return "round";
            }
            return "angular";
        }

        public static string Match(Shape shape)
        {
            return shape switch
            {
                Circle c => $"circle {c.Radius}",
                Rectangle r => $"rect {r.Width}x{r.Height}",
                _ => "unknown",
            };
        }
    }
}
