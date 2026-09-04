#ifndef STORE_H
#define STORE_H

#include <map>
#include <memory>
#include <string>
#include <vector>

template <typename T>
class Store {
public:
    void put(const std::string& key, T value);
    T get(const std::string& key) const;
    std::vector<std::string> keys() const;
private:
    std::map<std::string, T> data_;
};

class Connection {
public:
    explicit Connection(const std::string& dsn);
    ~Connection();
    Connection(const Connection&) = delete;
    Connection& operator=(const Connection&) = delete;
    std::string query(const std::string& sql);
private:
    std::string dsn_;
};

#endif
