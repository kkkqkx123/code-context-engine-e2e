export class User {
  name: string;
  age: number;

  constructor(name: string, age: number) {
    this.name = name;
    this.age = age;
  }

  greet(): string {
    return `Hello, ${this.name}!`;
  }
}

export function loadUser(name: string): User {
  return new User(name, 30);
}
