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

def load_user(name)
  User.new(name, 30)
end
