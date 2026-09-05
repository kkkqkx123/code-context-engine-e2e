namespace MyApp.Models
{
    public class Visibility
    {
        public string PubField = "public";
        internal string InternalField = "internal";
        protected string ProtectedField = "protected";
        private string PrivateField = "private";

        public string GetPublic()
        {
            return PubField;
        }

        internal string GetInternal()
        {
            return InternalField;
        }

        protected string GetProtected()
        {
            return ProtectedField;
        }

        private string GetPrivate()
        {
            return PrivateField;
        }

        public string Describe()
        {
            return GetPublic() + GetInternal() + GetProtected() + GetPrivate();
        }
    }
}
