export function combine(a: number, b: number): number;
export function combine(a: string, b: string): string;
export function combine(a: number, b: string): string;
export function combine(a: number | string, b: number | string): number | string {
  if (typeof a === "number" && typeof b === "number") {
    return a + b;
  }
  return String(a) + String(b);
}

export function run(): string {
  const ints = combine(1, 2);
  const strs = combine("a", "b");
  const mixed = combine(1, "b");
  return `${ints}${strs}${mixed}`;
}
