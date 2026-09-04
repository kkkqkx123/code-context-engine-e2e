#include <string>
#include <vector>
#include <map>

class Calculator {
public:
    int add(int a, int b) {
        return a + b;
    }

    std::string label() {
        return "calculator";
    }
};

template <typename T>
T identity(T x) {
    return x;
}

template <typename K, typename V>
std::map<K, V> to_map(K key, V value) {
    return std::map<K, V>{{key, value}};
}

int process_items(const std::vector<int>& items) {
    int total = 0;
    for (auto& elem : items) {
        total += elem;
    }
    return total;
}

int main() {
    auto count = 42;
    auto name = std::string("hello");
    decltype(count) other = 7;
    int explicit_val = 10;
    Calculator calc = Calculator();
    std::string greeting = std::string("hi");
    std::vector<int> numbers = {1, 2, 3};
    int doubled = identity(21);
    std::string same = identity(std::string("x"));
    auto total = process_items(numbers);
    return 0;
}
