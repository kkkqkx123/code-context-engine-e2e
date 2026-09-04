const { loadUser } = require("./models");

function renderGreeting(user) {
  return user.greet();
}

function main() {
  const user = loadUser("Alice");
  console.log(renderGreeting(user));
  console.log(user.greet());
}

module.exports = { renderGreeting, main };
