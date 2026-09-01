package main

import (
	"fmt"
	"strings"
)

func add(a, b int) int {
	return a + b
}

func repeat(s string, n int) string {
	return strings.Repeat(s, n)
}

func main() {
	sum := add(1, 2)
	fmt.Println("Sum:", sum)
	result := repeat("hello", 3)
	fmt.Println("Repeated:", result)
}
