interface Container<T> {
    value: T;
    duplicate(): Container<T>;
}

function identity<T>(x: T): T {
    return x;
}

function wrapInArray<T>(item: T): T[] {
    return [item];
}

function first<T>(arr: T[]): T | undefined {
    return arr[0];
}

interface Pair<A, B> {
    first: A;
    second: B;
}

function makePair<A, B>(first: A, second: B): Pair<A, B> {
    return { first, second };
}

function swap<A, B>(pair: Pair<A, B>): Pair<B, A> {
    return { first: pair.second, second: pair.first };
}

function collectToMap<K extends string, V>(items: Array<[K, V]>): Map<K, V> {
    const map = new Map<K, V>();
    for (const [k, v] of items) {
        map.set(k, v);
    }
    return map;
}

function processNested(items: Array<Map<string, number>>): number {
    let total = 0;
    for (const map of items) {
        for (const value of map.values()) {
            total += value;
        }
    }
    return total;
}

const container: Container<string> = {
    value: "hello",
    duplicate() {
        return { value: this.value };
    },
};

const pair = makePair(42, "answer");
const swapped = swap(pair);
const wrapped = wrapInArray(10);
const items: Array<[string, number]> = [["a", 1], ["b", 2]];
const map = collectToMap(items);
