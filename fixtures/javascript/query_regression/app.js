import x from "mod-default";
import * as ns from "mod-ns";
import { a as b, c } from "mod-named";
import "mod-side";
export { x } from "mod-reexport";
const lazy = await import("mod-dynamic");
const cjs = require("mod-cjs");

class Foo {
  constructor() {
    this.ready = true;
  }
  run() {
    return helper(this.ready);
  }
}

const f = new Foo();
f.run();
promise.then(done).catch(fail);
maybe?.call();
