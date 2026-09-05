String handleIsNegated(Object value) {
  if (value is! String) {
    return 'not a string';
  }
  return value.toUpperCase();
}

String handleNotNull(String? value) {
  if (value is! String) {
    return 'missing';
  }
  return value;
}

String handleNullCheck(String? value) {
  if (value != null) {
    return value;
  }
  return 'missing';
}

void main() {
  print(handleIsNegated(42));
  print(handleNotNull('hello'));
  print(handleNullCheck('hello'));
}
