type Mapper<T, U> = (value: T) => U;

const toLabel: Mapper<number, string> = (n) => `n=${n}`;

function applyTwice(value: number, fn: (x: number) => number): number {
    return fn(fn(value));
}

function fetchGreeting(name: string): Promise<string> {
    return Promise.resolve(`hello ${name}`);
}

const doubled = applyTwice(21, (x) => x * 2);
const chained = fetchGreeting("ada").then((greeting) => greeting.length);

export { toLabel, applyTwice, fetchGreeting, doubled, chained };
