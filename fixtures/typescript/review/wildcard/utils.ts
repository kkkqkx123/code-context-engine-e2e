export function alpha(): string {
  return "alpha";
}

export function beta(): string {
  return "beta";
}

function internalHelper(): string {
  return "helper";
}

export { internalHelper as gamma };
