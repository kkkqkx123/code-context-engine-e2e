local app_name = "demo"
local max_retries = 3
local verbose = true

local function log_message(msg)
  return "[" .. app_name .. "] " .. msg
end

local counter = 0
local greeting = log_message("start")
local helper = require("helper")

return {
  counter = counter,
  greeting = greeting,
  formatted = helper.format(app_name),
}
