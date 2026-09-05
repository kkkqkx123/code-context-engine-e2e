using System;
using System.Collections.Generic;
using System.Linq;

namespace LambdaApp
{
    public class LambdaDemo
    {
        public static string ApplyTwice(Func<int, int> fn, int value)
        {
            return fn(fn(value)).ToString();
        }

        public static void Main()
        {
            Func<int, string> toLabel = x => $"n={x}";
            var doubled = ApplyTwice(x => x * 2, 21);
            var labels = new List<int> { 1, 2, 3 }.Select(x => x.ToString()).ToList();
            Console.WriteLine($"{toLabel(1)} {doubled} {labels.Count}");
        }
    }
}
