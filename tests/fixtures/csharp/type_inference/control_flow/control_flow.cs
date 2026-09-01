using System;
using System.Collections.Generic;

public class ControlFlowDemo
{
    public static string HandleIsCheck(object obj)
    {
        if (obj is string s)
        {
            return s.ToUpper();
        }
        else if (obj is int n)
        {
            return $"number: {n}";
        }
        return "unknown";
    }

    public static string HandleTryCatch()
    {
        try
        {
            int.Parse("not_a_number");
            return "parsed";
        }
        catch (FormatException e)
        {
            return $"error: {e.Message}";
        }
        catch (Exception e)
        {
            return $"general error: {e.Message}";
        }
    }

    public static string HandleTryCatchWhen()
    {
        try
        {
            throw new ArgumentException("test");
        }
        catch (ArgumentException ex) when (ex.Message.Contains("test"))
        {
            return $"filtered: {ex.Message}";
        }
    }

    public static string HandleTryFinally()
    {
        string result = "default";
        try
        {
            result = "modified";
        }
        finally
        {
            result = result + " (finally)";
        }
        return result;
    }

    public static string HandleMultipleCatch()
    {
        try
        {
            throw new InvalidOperationException("test");
        }
        catch (InvalidOperationException e)
        {
            return $"invalid op: {e.Message}";
        }
        catch (ArgumentException e)
        {
            return $"arg: {e.Message}";
        }
        catch (Exception e)
        {
            return $"general: {e.Message}";
        }
    }

    public static void Main()
    {
        Console.WriteLine(HandleIsCheck("hello"));
        Console.WriteLine(HandleIsCheck(42));
        Console.WriteLine(HandleTryCatch());
        Console.WriteLine(HandleTryCatchWhen());
        Console.WriteLine(HandleTryFinally());
        Console.WriteLine(HandleMultipleCatch());
    }
}
