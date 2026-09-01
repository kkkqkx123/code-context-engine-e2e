<?php

class Calculator
{
    public function add(int $a, int $b): int
    {
        return $a + $b;
    }

    public function multiply(int $a, int $b): int
    {
        return $a * $b;
    }
}

$calc = new Calculator();
echo $calc->add(1, 2) . "\n";
echo $calc->multiply(3, 4) . "\n";
