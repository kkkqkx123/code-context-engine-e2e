sealed class Result {
  const Result();
}

class Success extends Result {
  final String value;
  Success(this.value);
}

class Failure extends Result {
  final String message;
  Failure(this.message);
}

String describe(Result result) {
  if (result is Success) {
    return result.value;
  }
  if (result is Failure) {
    return result.message;
  }
  return 'pending';
}

void main() {
  print(describe(Success('done')));
  print(describe(Failure('oops')));
}
