sealed class Result {
  const Result();
}

class Success extends Result {
  String value = '';
}

class Failure extends Result {
  String message = '';
}

String handleIs(Object value) {
  if (value is String) {
    return value.toUpperCase();
  }
  if (value is int) {
    return 'number: $value';
  }
  return 'unknown';
}

String handleResult(Result result) {
  if (result is Success) {
    return result.value;
  }
  if (result is Failure) {
    return result.message;
  }
  return 'pending';
}

String handleKind(String kind, Object payload) {
  if (kind == 'text') {
    return payload.toString();
  }
  return 'other';
}

void main() {
  print(handleIs('hello'));
  print(handleIs(42));
  print(handleResult(Success()));
  print(handleKind('text', 'payload'));
}
