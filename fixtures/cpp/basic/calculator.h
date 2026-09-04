#ifndef CALCULATOR_H
#define CALCULATOR_H

#include <string>

class Calculator {
public:
    int add(int a, int b);
    int multiply(int a, int b);
    std::string label();
};

int add(int a, int b);

#endif
