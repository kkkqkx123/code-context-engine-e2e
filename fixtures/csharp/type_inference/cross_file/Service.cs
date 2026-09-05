using MyApp.Models;

namespace MyApp.Services
{
    public class Service
    {
        public string RenderGreeting(User user)
        {
            return user.Greet();
        }

        public void Main()
        {
            User user = UserFactory.CreateUser("Alice");
            Console.WriteLine(RenderGreeting(user));
            Console.WriteLine(user.Greet());
        }
    }
}
