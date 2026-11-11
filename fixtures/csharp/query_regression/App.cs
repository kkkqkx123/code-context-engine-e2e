using System;
using System.IO;
using Foo = Bar.Baz;
using static System.Math;
global using System.Linq;
global using static System.Text;
using L = System.Collections.Generic.List<int>;
using SB = System.Text.StringBuilder;

namespace Demo
{
    using Inner.Space;

    public class Base
    {
        public void M() { }
        public virtual void V() { }
    }

    public class Worker : Base
    {
        public void H() { }
        public void G() { this.H(); }
        public void F() { base.M(); }
        public void Run() { Console.WriteLine(Max(1, 2)); }
        public void Build()
        {
            var w = new Widget();
            var empty = new();
            chain.First().Second();
            target?.M();
            Generic<int>();
        }

        public Worker(int x) : base(x) { }
        public override void V() { this.H(); }
    }
}
