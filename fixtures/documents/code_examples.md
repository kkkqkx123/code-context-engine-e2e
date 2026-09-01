# Code Examples

This document contains code blocks of various sizes to test embedding text generation.

## Small Rust Block

```rust
fn main() {
    println!("Hello, world!");
}
```

## Large Rust Block

```rust
use std::collections::HashMap;

struct AppConfig {
    host: String,
    port: u16,
    workers: usize,
}

impl AppConfig {
    fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            workers: 4,
        }
    }

    fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

trait Logger {
    fn log(&self, level: LogLevel, message: &str);
}

mod network {
    pub fn connect(addr: &str) -> Result<(), String> {
        Ok(())
    }
}
```

## Small Python Block

```python
def greet(name):
    return f"Hello, {name}"
```

## Large Python Block

```python
from typing import List, Optional
from dataclasses import dataclass


@dataclass
class User:
    name: str
    email: str
    age: int

    def is_adult(self) -> bool:
        return self.age >= 18


class UserRepository:
    def __init__(self):
        self._users: List[User] = []

    def add(self, user: User) -> None:
        self._users.append(user)

    def find_by_name(self, name: str) -> Optional[User]:
        for user in self._users:
            if user.name == name:
                return user
        return None

    def all_adults(self) -> List[User]:
        return [u for u in self._users if u.is_adult()]


def create_admin(name: str, email: str) -> User:
    return User(name=name, email=email, age=30)
```

## Unsupported Language Block

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4
max_connections = 100

[database]
host = "localhost"
port = 5432
name = "cce_production"
user = "app_user"
pool_size = 20

[cache]
enabled = true
ttl_seconds = 3600
max_size_mb = 512
strategy = "lru"

[logging]
level = "info"
format = "json"
output = "stdout"
file = "/var/log/cce/app.log"

[rate_limiting]
enabled = true
requests_per_minute = 1000
burst_size = 50
```

## Plain Text Block

```
This is a plain text block
with multiple lines
that should be treated as-is
when generating embedding text.
```
