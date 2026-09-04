<?php

namespace App\Models;

trait Timestampable
{
    public function touch(): string
    {
        return date("c");
    }
}

class User
{
    use Timestampable;

    public function __construct(public string $name, public int $age) {}

    public function greet(): string
    {
        return "Hello, {$this->name}!";
    }
}
