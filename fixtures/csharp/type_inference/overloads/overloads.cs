namespace OverloadApp
{
    public class Overloads
    {
        public int Combine(int a, int b)
        {
            return a + b;
        }

        public string Combine(string a, string b)
        {
            return a + b;
        }

        public string Combine(int a, string b)
        {
            return a + b;
        }

        public string Run()
        {
            int ints = Combine(1, 2);
            string strs = Combine("a", "b");
            string mixed = Combine(1, "b");
            return $"{ints}{strs}{mixed}";
        }
    }
}
