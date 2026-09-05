import java.util.List;
import java.util.function.Function;
import java.util.stream.Collectors;

public class LambdaDemo {

    public static int applyTwice(Function<Integer, Integer> fn, int value) {
        return fn.apply(fn.apply(value));
    }

    public static void main(String[] args) {
        Function<String, Integer> lengthOf = s -> s.length();
        int doubled = applyTwice(x -> x * 2, 21);
        List<String> labels = List.of(1, 2, 3).stream()
                .map(n -> "n=" + n)
                .collect(Collectors.toList());
        System.out.println(lengthOf.apply("hello") + doubled + labels.size());
    }
}
