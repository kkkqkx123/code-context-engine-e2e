#include <string>
#include <sstream>

class Overloads {
public:
    int combine(int a, int b) {
        return a + b;
    }

    std::string combine(const std::string& a, const std::string& b) {
        return a + b;
    }

    std::string combine(int a, const std::string& b) {
        std::ostringstream oss;
        oss << a << b;
        return oss.str();
    }

    std::string run() {
        int ints = combine(1, 2);
        std::string strs = combine("a", "b");
        std::string mixed = combine(1, "b");
        std::ostringstream oss;
        oss << ints << strs << mixed;
        return oss.str();
    }
};
