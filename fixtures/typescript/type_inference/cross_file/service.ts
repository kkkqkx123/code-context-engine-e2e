import { User, loadUser } from "./models";

export function renderGreeting(user: User): string {
  return user.greet();
}

export function main(): void {
  const user = loadUser("Alice");
  console.log(renderGreeting(user));
  console.log(user.greet());
}
