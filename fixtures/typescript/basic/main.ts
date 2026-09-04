import { Calculator, createCalculator } from "./calculator";
import { reverse, wordCount } from "./string_utils";

export function process(a: number, b: number): number {
  const calc = createCalculator();
  const sum = calc.add(a, b);
  return calc.multiply(sum, 2);
}

function main(): void {
  const calc = new Calculator();
  const sum = calc.add(1, 2);
  const product = calc.multiply(sum, 3);
  const text = reverse("hello");
  const count = wordCount("hello world");
  console.log(product, text, count, process(1, 2));
}

main();
