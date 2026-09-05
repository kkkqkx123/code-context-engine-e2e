import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class VarInference {

    public static void main(String[] args) {
        var names = new ArrayList<String>();
        names.add("ada");
        var scores = new HashMap<String, Integer>();
        scores.put("ada", 10);
        var first = names.get(0);
        var doubled = first + first;
        System.out.println(doubled + scores.size());
    }
}
