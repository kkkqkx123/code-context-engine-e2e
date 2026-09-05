class User {
  final String name;
  final int age;

  User(this.name, this.age);

  String greet() => 'Hello, $name!';
}

User loadUser(String name) => User(name, 30);
