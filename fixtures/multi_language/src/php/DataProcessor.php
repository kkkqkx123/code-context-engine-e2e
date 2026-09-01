<?php

class DataProcessor
{
    private array $items = [];

    public function addItem(string $item): void
    {
        $this->items[] = $item;
    }

    public function getItems(): array
    {
        return $this->items;
    }

    public function count(): int
    {
        return count($this->items);
    }
}
