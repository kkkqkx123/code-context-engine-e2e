function processValues(values: number[] | null): number {
    if (values !== null) {
        const buffer: number[] = [];
        for (const item of values) {
            if (item < 0) {
                continue;
            }
            buffer.push(item);
        }

        const borrowed = buffer.length;
        const _ = borrowed;
        const shifted = 1 << 2;

        let outcome: number;
        while (true) {
            if (buffer.length === 0) {
                outcome = buffer.length;
                break;
            } else {
                outcome = buffer.length;
                break;
            }
        }

        return outcome;
    } else {
        throw new Error("missing values");
    }
}
