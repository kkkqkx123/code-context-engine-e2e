import java.util.List;
import static java.lang.Math.max;
import java.util.*;

class Main {
    private List<String> items;

    Main() {
        this.items = new java.util.ArrayList<>();
    }

    Main run() {
        var capped = max(items.size(), 10);
        return this;
    }

    void finish() {
        Runnable r = Main::run;
        System.out.println(super.toString());
    }

    public static void main(String[] args) {
        var m = new Main();
        m.run().finish();
    }
}
