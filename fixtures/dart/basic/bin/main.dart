import '../lib/calculator.dart';

int process(int a, int b) {
  final calc = Calculator();
  final sum = calc.add(a, b);
  return calc.multiply(sum, 2);
}

void main() {
  final calc = Calculator();
  final sum = calc.add(1, 2);
  final product = calc.multiply(sum, 3);
  final text = reverse('hello');
  final count = wordCount('hello world');
  print('$product $text $count ${process(1, 2)}');
}
