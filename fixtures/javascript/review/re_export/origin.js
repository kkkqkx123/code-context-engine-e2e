export function formatName(name) {
    return `hello ${name}`;
}

export class Greeter {
    greet(name) {
        return formatName(name);
    }
}
