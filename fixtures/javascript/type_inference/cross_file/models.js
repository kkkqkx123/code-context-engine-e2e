class User {
  constructor(name, age) {
    this.name = name;
    this.age = age;
  }

  greet() {
    return `Hello, ${this.name}!`;
  }
}

function loadUser(name) {
  return new User(name, 30);
}

module.exports = { User, loadUser };
