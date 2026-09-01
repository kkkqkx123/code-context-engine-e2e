public class Helper {
    public static int process() {
        int x = internal();
        return x * 2;
    }

    private static int internal() {
        return 21;
    }
}

