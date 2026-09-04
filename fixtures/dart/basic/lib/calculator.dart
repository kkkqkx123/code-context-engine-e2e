class Calculator {
  int add(int a, int b) => a + b;

  int multiply(int a, int b) => a * b;
}

String reverse(String input) => input.split('').reversed.join();

int wordCount(String input) => input.trim().split(RegExp(r'\s+')).length;
