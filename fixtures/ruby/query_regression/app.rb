require "foo"
require_relative "./bar"
load "baz.rb"
autoload :Helper, "helper"
gem "blueberry"

class Animal
end

class Lone < Base
end

class Dog < Animal
  include Fetch
  extend Tricks
  prepend Loud

  def speak
    super
    fetch.bone
    @leash&.clip
    puts "woof"
  end
end

Dog.new
