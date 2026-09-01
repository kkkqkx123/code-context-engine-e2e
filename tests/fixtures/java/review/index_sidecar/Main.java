import java.util.List;
import java.util.ArrayList;

public class Main {

    public static int processValues(List<Integer> values) throws Exception {
        if (values != null) {
            ArrayList<Integer> buffer = new ArrayList<>();
            for (Integer item : values) {
                if (item < 0) {
                    continue;
                }
                buffer.add(item);
            }

            int borrowed = buffer.size();
            int _ = borrowed;
            int shifted = 1 << 2;

            int outcome;
            while (true) {
                if (buffer.size() == 0) {
                    outcome = buffer.size();
                    break;
                } else {
                    outcome = buffer.size();
                    break;
                }
            }

            return outcome;
        } else {
            throw new Exception("missing values");
        }
    }

}
