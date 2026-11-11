#include "helper.h"
#include <string>

using namespace mylib;
using extlib::Widget;

namespace mylib {

class Gadget {
public:
    Gadget();
    ~Gadget();
    void start();
    Gadget& operator=(const Gadget& other);
};

template<typename T>
class Box {
public:
    explicit Box(T v) : value(v) {}
    T value;
};

template<typename T>
T identity(T x) {
    return x;
}

Gadget::Gadget() {}
Gadget::~Gadget() {}
void Gadget::start() {}

}

int main() {
    std::string label = std::string("demo");
    auto w = new Gadget();
    w->start();
    auto v = identity<int>(42);
    auto b = Box<int>(v);
    return helper();
}
