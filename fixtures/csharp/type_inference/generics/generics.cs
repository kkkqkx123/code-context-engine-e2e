using System;
using System.Collections.Generic;
using System.Linq;

public class GenericsDemo
{
    public static string PrintPair<T, U>(T a, U b)
    {
        return $"{a}: {b}";
    }

    public static List<T> WrapInList<T>(T item)
    {
        return new List<T> { item };
    }

    public static T Identity<T>(T x)
    {
        return x;
    }

    public static Dictionary<K, V> ToMap<K, V>(K key, V value) where K : notnull
    {
        return new Dictionary<K, V> { { key, value } };
    }

    public class Container<T>
    {
        public T Value { get; }

        public Container(T value)
        {
            Value = value;
        }

        public Container<T> Duplicate()
        {
            return new Container<T>(Value);
        }
    }

    public class Pair<A, B>
    {
        public A First { get; }
        public B Second { get; }

        public Pair(A first, B second)
        {
            First = first;
            Second = second;
        }

        public Pair<B, A> Swap()
        {
            return new Pair<B, A>(Second, First);
        }
    }

    public static T Max<T>(T a, T b) where T : IComparable<T>
    {
        return a.CompareTo(b) >= 0 ? a : b;
    }

    public static void Main()
    {
        string pair = PrintPair(42, "hello");
        List<string> list = WrapInList("item");
        string id = Identity("test");
        var map = ToMap("key", 100);
        var container = new Container<string>("value");
        var dup = container.Duplicate();
        var p = new Pair<int, string>(1, "one");
        var swapped = p.Swap();
        int biggest = Max(10, 20);
    }
}
