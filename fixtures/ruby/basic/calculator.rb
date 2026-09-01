class Calculator
  def add(a, b)
    a + b
  end

  def multiply(a, b)
    a * b
  end
end

calc = Calculator.new
puts calc.add(1, 2)
puts calc.multiply(3, 4)
