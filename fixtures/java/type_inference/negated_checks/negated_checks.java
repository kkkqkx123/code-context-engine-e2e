public class NegatedChecks {

    public static String handleNotInstance(Object obj) {
        if (!(obj instanceof String)) {
            return "not a string";
        }
        return ((String) obj).toUpperCase();
    }

    public static String handleNull(String value) {
        if (value != null) {
            return value;
        }
        return "missing";
    }

    public static void main(String[] args) {
        System.out.println(handleNotInstance(42));
        System.out.println(handleNull("hello"));
    }
}
