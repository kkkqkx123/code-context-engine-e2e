namespace MyApp.Models
{
    public class User
    {
        public string Name { get; }
        public int Age { get; }

        public User(string name, int age)
        {
            Name = name;
            Age = age;
        }

        public string Greet()
        {
            return $"Hello, {Name}!";
        }
    }

    public static class UserFactory
    {
        public static User CreateUser(string name)
        {
            return new User(name, 30);
        }
    }
}
