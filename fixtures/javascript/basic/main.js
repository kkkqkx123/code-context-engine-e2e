const { Calculator, createCalculator } = require("./calculator");
const { reverse, wordCount } = require("./stringUtils");

function process(a, b) {
  const calc = createCalculator();
  const sum = calc.add(a, b);
  return calc.multiply(sum, 2);
}

function main() {
  const calc = new Calculator();
  const sum = calc.add(1, 2);
  const product = calc.multiply(sum, 3);
  const text = reverse("hello");
  const count = wordCount("hello world");
  console.log(product, text, count, process(1, 2));
}

main();

module.exports = { process, main };
