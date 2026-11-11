package main

import "fmt"

import (
	"os"
	alias "example.com/foo"
	. "example.com/dot"
	_ "example.com/blank"
)

type Reader interface {
	Reader
}

type Wrapper struct {
	Inner
	*Ptr
	pkg.Type
}

func main() {
	fmt.Println(os.Args[0])
	go serve()
	defer cleanup()
	chain.First().Second()
}
