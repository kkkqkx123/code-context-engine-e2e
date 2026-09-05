class Overloads {
  int combine(int a, int b) => a + b;

  String combineStrings(String a, String b) => a + b;

  String combineMixed(int a, String b) => '$a$b';

  String run() {
    final ints = combine(1, 2);
    final strs = combineStrings('a', 'b');
    final mixed = combineMixed(1, 'b');
    return '$ints$strs$mixed';
  }
}
