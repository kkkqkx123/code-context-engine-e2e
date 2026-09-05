<?php

class Overloads {
    public function combineInts(int $a, int $b): int {
        return $a + $b;
    }

    public function combineStrings(string $a, string $b): string {
        return $a . $b;
    }

    public function combineMixed(int $a, string $b): string {
        return $a . $b;
    }

    public function run(): string {
        $ints = $this->combineInts(1, 2);
        $strs = $this->combineStrings("a", "b");
        $mixed = $this->combineMixed(1, "b");
        return $ints . $strs . $mixed;
    }
}
