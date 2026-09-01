import java.util.*;

public class ControlFlowDemo {

    public static String handleInstanceof(Object obj) {
        if (obj instanceof String) {
            return ((String) obj).toUpperCase();
        } else if (obj instanceof Integer) {
            return "number: " + obj;
        }
        return "unknown";
    }

    public static String handleTryCatch() {
        try {
            Integer.parseInt("not_a_number");
            return "parsed";
        } catch (NumberFormatException e) {
            return "error: " + e.getMessage();
        } catch (Exception e) {
            return "general error";
        }
    }

    public static String handleMultiCatch() {
        try {
            String s = null;
            s.length();
            return s;
        } catch (NullPointerException | IndexOutOfBoundsException e) {
            return "null or index error";
        }
    }

    public static String handleTryFinally() {
        String result = "default";
        try {
            result = "modified";
        } finally {
            result = result + " (finally)";
        }
        return result;
    }

    public static String handleFinalCatch() {
        try {
            throw new RuntimeException("test");
        } catch (final RuntimeException e) {
            return e.getMessage();
        }
    }

    public static void main(String[] args) {
        System.out.println(handleInstanceof("hello"));
        System.out.println(handleInstanceof(42));
        System.out.println(handleTryCatch());
        System.out.println(handleMultiCatch());
        System.out.println(handleTryFinally());
        System.out.println(handleFinalCatch());
    }
}
