function handleNotNull(value: string | null): string {
    if (value !== null) {
        return value.toUpperCase();
    }
    return "missing";
}

function handleNegated(value: string | number): string {
    if (typeof value !== "string") {
        return value.toFixed(1);
    }
    return value;
}

function handleFalsy(value: string | undefined): string {
    if (!value) {
        return "empty";
    }
    return value;
}

const a = handleNotNull("hello");
const b = handleNegated(42);
const c = handleFalsy(undefined);

export { handleNotNull, handleNegated, handleFalsy, a, b, c };
