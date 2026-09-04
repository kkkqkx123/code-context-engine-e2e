require_relative "user"

module DynamicHelpers
  [:shout, :whisper].each do |kind|
    define_method(kind) do |text|
      kind == :shout ? text.upcase : text.downcase
    end
  end
end

class Bot
  include DynamicHelpers
end

user = User.new("Alice")
admin = Admin.new("Root")
puts user.greet
puts admin.greet
puts Bot.new.shout("hello")
