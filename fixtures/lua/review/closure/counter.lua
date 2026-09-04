local Counter = {}
Counter.__index = Counter

function Counter.new(start)
  local self = setmetatable({ value = start or 0 }, Counter)
  return self
end

function Counter:next(step)
  self.value = self.value + (step or 1)
  return self.value
end

local function make_adder(base)
  return function(x)
    return base + x
  end
end

local add_five = make_adder(5)

return {
  counter = Counter.new(10),
  added = add_five(7),
}
