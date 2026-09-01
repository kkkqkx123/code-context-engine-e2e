package main

import "fmt"

type Stringer interface {
	String() string
}

type Named interface {
	Name() string
}

type Formatter interface {
	Stringer
	Format() string
}

type Person struct {
	Name string
	Age  int
}

func (p Person) String() string {
	return fmt.Sprintf("%s (age %d)", p.Name, p.Age)
}

func (p Person) Name() string {
	return p.Name
}

func (p Person) Format() string {
	return fmt.Sprintf("Person{Name: %s, Age: %d}", p.Name, p.Age)
}

func processStringer(s Stringer) string {
	return s.String()
}

func processNamed(n Named) string {
	return n.Name()
}

func wrapInSlice[T any](item T) []T {
	return []T{item}
}

func first[T any](items []T) T {
	return items[0]
}

func main() {
	p := Person{Name: "Alice", Age: 30}
	var s Stringer = p
	var n Named = p
	var f Formatter = p

	fmt.Println(processStringer(s))
	fmt.Println(processNamed(n))
	fmt.Println(f.Format())

	wrapped := wrapInSlice("hello")
	firstItem := first([]int{1, 2, 3})
	fmt.Println(wrapped, firstItem)
}
