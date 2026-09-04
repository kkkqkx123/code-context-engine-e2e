mixin Logger on Service {
  void log(String message) {
    print('[${name}] $message');
  }
}

abstract class Service {
  String get name;

  Future<String> start();
}
