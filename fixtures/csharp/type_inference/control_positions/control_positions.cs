namespace PositionApp
{
    public class ControlPositions
    {
        public static string HandleWhile(object value)
        {
            var current = value;
            while (current is string) {
                current = ((string)current).ToUpper();
            }
            return current.ToString();
        }

        public static string HandleElseIf(object value)
        {
            if (value is string) {
                return (string)value;
            } else if (value is int) {
                return "number";
            }
            return "other";
        }
    }
}
