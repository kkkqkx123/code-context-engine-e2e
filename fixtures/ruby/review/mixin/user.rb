module Greetable
  def greet
    "Hello, #{name}!"
  end
end

class User
  include Greetable

  attr_reader :name

  def initialize(name)
    @name = name
  end
end

class Admin < User
  def greet
    "Admin: #{super}"
  end
end
