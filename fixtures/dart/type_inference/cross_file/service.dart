import 'user.dart';

String renderGreeting(User user) => user.greet();

void main() {
  final user = loadUser('Alice');
  print(renderGreeting(user));
  print(user.greet());
}
