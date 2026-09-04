import 'service.dart';

class ApiService extends Service with Logger {
  @override
  String get name => 'api';

  @override
  Future<String> start() async {
    log('starting');
    await Future.delayed(Duration(milliseconds: 1));
    return 'ready';
  }
}

Future<void> main() async {
  final service = ApiService();
  final status = await service.start();
  service.log(status);
}
