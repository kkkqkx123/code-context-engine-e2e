local vars = require("vars")

local function describe()
  return vars.greeting .. " retries=" .. tostring(3)
end

print(describe())
