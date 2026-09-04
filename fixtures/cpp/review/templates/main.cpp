#include <iostream>
#include "store.h"

template <typename T>
void Store<T>::put(const std::string& key, T value) {
    data_[key] = value;
}

template <typename T>
T Store<T>::get(const std::string& key) const {
    return data_.at(key);
}

template <typename T>
std::vector<std::string> Store<T>::keys() const {
    std::vector<std::string> out;
    for (const auto& [k, _] : data_) {
        out.push_back(k);
    }
    return out;
}

Connection::Connection(const std::string& dsn) : dsn_(dsn) {
    std::cout << "open " << dsn_ << std::endl;
}

Connection::~Connection() {
    std::cout << "close " << dsn_ << std::endl;
}

std::string Connection::query(const std::string& sql) {
    return "result:" + sql;
}

int main() {
    Store<int> ints;
    ints.put("one", 1);
    std::unique_ptr<Connection> conn = std::make_unique<Connection>("db");
    std::cout << ints.get("one") << conn->query("select 1") << std::endl;
    return 0;
}
