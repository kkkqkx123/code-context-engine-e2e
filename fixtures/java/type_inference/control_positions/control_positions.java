public class ControlPositions {

    public static String handleWhile(Object value) {
        Object current = value;
        while (current instanceof String) {
            current = ((String) current).toUpperCase();
        }
        return current.toString();
    }

    public static String handleElseIf(Object value) {
        if (value instanceof String) {
            return (String) value;
        } else if (value instanceof Integer) {
            return "number: " + value;
        }
        return "other";
    }

    public static void main(String[] args) {
        System.out.println(handleWhile("hi"));
        System.out.println(handleElseIf(42));
    }
}
