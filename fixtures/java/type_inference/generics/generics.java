import java.util.*;

public class GenericsDemo {

    public static <T, U> String printPair(T a, U b) {
        return a.toString() + ": " + b.toString();
    }

    public static <T> List<T> wrapInList(T item) {
        List<T> list = new ArrayList<>();
        list.add(item);
        return list;
    }

    public static <T> T identity(T x) {
        return x;
    }

    public static <K, V> Map<K, V> toMap(K key, V value) {
        Map<K, V> map = new HashMap<>();
        map.put(key, value);
        return map;
    }

    static class Container<T> {
        private T value;

        public Container(T value) {
            this.value = value;
        }

        public T getValue() {
            return value;
        }

        public Container<T> duplicate() {
            return new Container<>(value);
        }
    }

    static class Pair<A, B> {
        private A first;
        private B second;

        public Pair(A first, B second) {
            this.first = first;
            this.second = second;
        }

        public A getFirst() { return first; }
        public B getSecond() { return second; }

        public Pair<B, A> swap() {
            return new Pair<>(second, first);
        }
    }

    public static <T extends Comparable<T>> T max(T a, T b) {
        return a.compareTo(b) >= 0 ? a : b;
    }

    public static void main(String[] args) {
        String pair = printPair(42, "hello");
        List<String> list = wrapInList("item");
        String id = identity("test");
        Map<String, Integer> map = toMap("key", 100);
        Container<String> container = new Container<>("value");
        Container<String> dup = container.duplicate();
        Pair<Integer, String> p = new Pair<>(1, "one");
        Pair<String, Integer> swapped = p.swap();
        Integer biggest = max(10, 20);
    }
}
