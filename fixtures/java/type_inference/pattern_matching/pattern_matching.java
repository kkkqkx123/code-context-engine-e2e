public class PatternMatching {

    public static String describe(Object obj) {
        if (obj instanceof String s) {
            return s.toUpperCase();
        }
        if (obj instanceof Integer n) {
            return "number: " + n;
        }
        return "unknown";
    }

    public static String matchShape(Object obj) {
        if (obj instanceof String s && s.length() > 3) {
            return "long: " + s;
        }
        return "other";
    }

    public static void main(String[] args) {
        System.out.println(describe("hello"));
        System.out.println(describe(42));
        System.out.println(matchShape("abcd"));
    }
}
