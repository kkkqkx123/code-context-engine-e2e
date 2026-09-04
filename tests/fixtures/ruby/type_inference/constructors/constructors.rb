class User
  attr_reader :name, :age

  def initialize(name, age)
    @name = name
    @age = age
  end

  def greet
    "Hello, #{@name}!"
  end
end

class Calculator
  # @return [Integer] the sum of both operands
  def add(a, b)
    a + b
  end

  # @return [String] upper-cased input
  def shout(text)
    text.upcase
  end
end

user = User.new("Alice", 30)
calc = Calculator.new()
total = calc.add(1, 2)
loud = calc.shout("hello")
puts user.greet
puts total
puts loud
