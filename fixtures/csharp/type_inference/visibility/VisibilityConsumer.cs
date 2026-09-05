using MyApp.Models;

namespace MyApp.Services
{
    public class VisibilityConsumer
    {
        public string ConsumePublic(Visibility v)
        {
            return v.GetPublic();
        }

        public string ConsumeDescribe(Visibility v)
        {
            return v.Describe();
        }
    }
}
