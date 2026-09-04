package app;

public class Overloads {
    public int combine(int a, int b) {
        return a + b;
    }

    public String combine(String a, String b) {
        return a + b;
    }

    public String combine(int a, String b) {
        return a + b;
    }

    public String run() {
        int ints = combine(1, 2);
        String strs = combine("a", "b");
        String mixed = combine(1, "b");
        return ints + strs + mixed;
    }
}
