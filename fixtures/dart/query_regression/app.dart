import "util.dart";
export "api.dart";
part "impl.dart";

class App {
  void run() {
    var svc = Service();
    svc.start();
  }
}
