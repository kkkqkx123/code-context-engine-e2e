local helper = require("helper")

local function greet(name)
  return helper.format(name)
end

return greet
