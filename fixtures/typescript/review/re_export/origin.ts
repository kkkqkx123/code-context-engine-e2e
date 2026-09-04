export function formatName(name: string): string {
  return `hello ${name}`;
}

export class Greeter {
  greet(name: string): string {
    return formatName(name);
  }
}
