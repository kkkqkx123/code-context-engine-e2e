String handleWhile(Object value) {
  var current = value;
  while (current is String) {
    current = current.toUpperCase();
  }
  return current.toString();
}

String handleElseIf(Object value) {
  if (value is String) {
    return value;
  } else if (value is int) {
    return 'number: $value';
  }
  return 'other';
}

void main() {
  print(handleWhile('hi'));
  print(handleElseIf(42));
}
