import type { Config } from "./config";
import { serve } from "./server";

interface Named extends Base {
  name: string;
}

class App extends Service implements Runnable {
  run(): void {
    serve();
  }
}

enum Mode { Fast, Slow }

export { App };
