interface User {
    name: string;
    age: number;
}

const user: User = { name: "ada", age: 36 };

const { name, age } = user;
const [first, second] = ["a", "b"];

function greet({ name, age }: User): string {
    return `${name} is ${age}`;
}

const scores = new Map<string, number>([["ada", 10]]);
let total = 0;
for (const [key, value] of scores) {
    total += value;
    console.log(key);
}

export { name, age, first, second, greet, total };
