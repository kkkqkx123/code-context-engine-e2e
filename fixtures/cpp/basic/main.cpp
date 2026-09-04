#include <iostream>
#include "calculator.h"

int main() {
    Calculator calc;
    int sum = calc.add(1, 2);
    int product = calc.multiply(sum, 3);
    int total = add(sum, product);
    std::cout << product << " " << total << " " << calc.label() << std::endl;
    return 0;
}
