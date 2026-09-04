local counter = require("counter")

local first = counter.counter:next(2)
local second = counter.counter:next()

print(first, second, counter.added)
