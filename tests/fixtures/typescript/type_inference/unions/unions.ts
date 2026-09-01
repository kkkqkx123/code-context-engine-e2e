type Success = { kind: "success"; value: string };
type Error = { kind: "error"; message: string };
type Pending = { kind: "pending" };
type Result = Success | Error | Pending;

function handleResult(result: Result): string {
    if (result.kind === "success") {
        return result.value;
    } else if (result.kind === "error") {
        return result.message;
    }
    return "pending";
}

function processValue(value: string | number | null | undefined): string {
    if (value === null || value === undefined) {
        return "empty";
    }
    if (typeof value === "string") {
        return value.toUpperCase();
    }
    return value.toFixed(2);
}

function narrowWithtypeof(x: unknown): string {
    if (typeof x === "string") {
        return x;
    }
    if (typeof x === "number") {
        return x.toString();
    }
    if (typeof x === "boolean") {
        return x ? "true" : "false";
    }
    return "unknown";
}

function checkInstance(x: unknown): string {
    if (x instanceof Array) {
        return `array of length ${x.length}`;
    }
    if (x instanceof Date) {
        return x.toISOString();
    }
    return "not array or date";
}

function inCheck(obj: Record<string, unknown>): string {
    if ("name" in obj) {
        return String(obj.name);
    }
    return "no name";
}

const result: Result = { kind: "success", value: "done" };
const output = handleResult(result);
const processed = processValue("hello");
const narrowed = narrowWithtypeof(42);
const instanced = checkInstance([1, 2, 3]);
