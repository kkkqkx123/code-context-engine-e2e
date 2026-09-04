class Calculator {
  add(a, b) {
    return a + b;
  }

  multiply(a, b) {
    return a * b;
  }
}

function createCalculator() {
  return new Calculator();
}

module.exports = { Calculator, createCalculator };
