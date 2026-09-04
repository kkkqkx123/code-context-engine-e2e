<?php

class User
{
    public string $name;
    public int $age;

    public function __construct(string $name, int $age)
    {
        $this->name = $name;
        $this->age = $age;
    }

    public function greet(): string
    {
        return "Hello, " . $this->name;
    }
}

class Calculator
{
    /**
     * @param int $a
     * @param int $b
     * @return int
     */
    public function add(int $a, int $b): int
    {
        return $a + $b;
    }

    /** @return string */
    public function shout(string $text): string
    {
        return strtoupper($text);
    }
}

/** @var string $greeting */
$greeting = "hello";

$user = new User("Alice", 30);
$calc = new Calculator();
/** @var int $total */
$total = $calc->add(1, 2);
echo $user->greet() . "\n";
echo $total . "\n";
echo $greeting . "\n";
