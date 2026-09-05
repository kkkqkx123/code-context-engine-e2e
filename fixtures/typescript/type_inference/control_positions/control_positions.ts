function handleWhile(value: string | number): string {
    let current = value;
    while (typeof current === "number") {
        current = current.toString();
    }
    return current;
}

function handleElseIf(value: unknown): string {
    if (typeof value === "string") {
        return value;
    } else if (value instanceof Array) {
        return `array:${value.length}`;
    }
    return "other";
}

function earlyReturn(value: string | number | null): string {
    if (value === null) {
        return "null";
    }
    if (typeof value === "number") {
        return value.toFixed(0);
    }
    return value.toUpperCase();
}

const w = handleWhile(1);
const e = handleElseIf([1, 2]);
const r = earlyReturn("hi");

export { handleWhile, handleElseIf, earlyReturn, w, e, r };
