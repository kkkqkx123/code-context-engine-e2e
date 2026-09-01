using System;

namespace CalculatorApp
{
    public class Calculator
    {
        public int Add(int a, int b)
        {
            return a + b;
        }

        public int Multiply(int a, int b)
        {
            return a * b;
        }
    }

    public class Program
    {
        public static void Main(string[] args)
        {
            var calc = new Calculator();
            int sum = calc.Add(1, 2);
            int product = calc.Multiply(sum, 3);
            Console.WriteLine($"Result: {product}");
        }
    }
}
