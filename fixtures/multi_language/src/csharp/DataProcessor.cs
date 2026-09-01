using System;
using System.Collections.Generic;

namespace MultiLanguage
{
    public class DataProcessor
    {
        private readonly List<string> _items = new List<string>();

        public void AddItem(string item)
        {
            _items.Add(item);
        }

        public IReadOnlyList<string> GetItems()
        {
            return _items.AsReadOnly();
        }

        public int Count()
        {
            return _items.Count;
        }
    }
}
