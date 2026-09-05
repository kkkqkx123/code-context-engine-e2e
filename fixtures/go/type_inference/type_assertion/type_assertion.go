package main

import "fmt"

type Greeter struct {
	Name string
}

func (g Greeter) Greet() string {
	return "hello " + g.Name
}

func describe(value any) string {
	if s, ok := value.(string); ok {
		return "string: " + s
	}
	if n, ok := value.(int); ok {
		return fmt.Sprintf("int: %d", n)
	}
	return "other"
}

func switchType(value any) string {
	switch v := value.(type) {
	case string:
		return "string: " + v
	case int:
		return fmt.Sprintf("int: %d", v)
	case Greeter:
		return v.Greet()
	default:
		return "unknown"
	}
}

func main() {
	fmt.Println(describe("hi"))
	fmt.Println(describe(42))
	fmt.Println(switchType(Greeter{Name: "ada"}))
}
