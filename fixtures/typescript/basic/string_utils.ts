export function reverse(input: string): string {
  return input.split("").reverse().join("");
}

export function wordCount(input: string): number {
  return input.trim().split(/\s+/).length;
}
