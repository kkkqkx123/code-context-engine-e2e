import { formatName, Greeter } from "./index";

export function render(name: string): string {
  const greeter = new Greeter();
  return greeter.greet(name) + "|" + formatName(name);
}
