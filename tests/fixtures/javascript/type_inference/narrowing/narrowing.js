function createUser(name, id) {
  return { kind: 'user', name, id };
}

function handleResult(result) {
  if (result == null) {
    return 'empty';
  }
  if (typeof result === 'string') {
    return result.toUpperCase();
  }
  if (typeof result === 'number') {
    return result.toFixed(2);
  }
  return 'unknown';
}

function handleShape(shape) {
  if (shape instanceof Array) {
    return `array of length ${shape.length}`;
  }
  if (shape instanceof Date) {
    return shape.toISOString();
  }
  return 'other';
}

function getName(obj) {
  if ('name' in obj) {
    return String(obj.name);
  }
  return 'no name';
}

function process(input) {
  if (input) {
    return handleResult(input);
  }
  return 'falsy';
}

const user = createUser('Alice', 1);
const a = handleResult('hello');
const b = handleResult(42);
const c = handleShape([1, 2, 3]);
const d = getName(user);
const e = process('go');
module.exports = { createUser, handleResult, handleShape, getName, process };
