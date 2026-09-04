import { alpha, beta } from "./barrel";

export function combined(): string {
  return alpha() + beta();
}
