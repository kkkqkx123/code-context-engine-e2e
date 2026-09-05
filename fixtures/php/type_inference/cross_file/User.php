<?php

namespace App\Models;

class User {
    private string $name;
    private int $age;

    public function __construct(string $name, int $age) {
        $this->name = $name;
        $this->age = $age;
    }

    public function getName(): string {
        return $this->name;
    }

    public function getAge(): int {
        return $this->age;
    }

    public function greet(): string {
        return "Hello, " . $this->name . "!";
    }
}

function loadUser(string $name): User {
    return new User($name, 30);
}
