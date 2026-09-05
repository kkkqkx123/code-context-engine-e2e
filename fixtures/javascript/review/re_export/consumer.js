import { formatName, Greeter } from './middle';

export function render(name) {
    const greeter = new Greeter();
    return `${greeter.greet(name)}|${formatName(name)}`;
}
