using System;
using System.Collections.Generic;

namespace LoggerApp
{
    public class Logger
    {
        private readonly List<string> _logs = new List<string>();

        public void Log(string message)
        {
            _logs.Add(message);
            Console.WriteLine(message);
        }

        public IReadOnlyList<string> GetLogs()
        {
            return _logs.AsReadOnly();
        }
    }
}
