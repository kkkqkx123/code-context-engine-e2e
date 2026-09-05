require_relative 'user'

def render_greeting(user)
  user.greet
end

user = load_user('Alice')
puts render_greeting(user)
puts user.greet
