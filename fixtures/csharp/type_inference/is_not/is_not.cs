namespace NegationApp
{
    public class IsNotDemo
    {
        public static string HandleIsNot(object obj)
        {
            if (obj is not string) {
                return "not a string";
            }
            return "is string";
        }

        public static string HandleNotNull(string value)
        {
            if (value is not null) {
                return value;
            }
            return "missing";
        }
    }
}
