class Container<T> {
  T? value;

  Container<T> duplicate() {
    final copy = Container<T>();
    copy.value = value;
    return copy;
  }
}

T identity<T>(T x) => x;

List<T> wrapInList<T>(T item) => [item];

String greet(String name, int age) => '$name is $age';

void main() {
  var count = 42;
  final name = 'hello';
  const tag = 'demo';
  String explicit = 'typed';
  var user = Container<String>('value');
  var dup = user.duplicate();
  var wrapped = wrapInList(10);
  var greeting = greet('Alice', 30);
  print('$count $name $tag $explicit $dup $wrapped $greeting');
}
