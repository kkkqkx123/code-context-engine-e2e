package main

import (
	"fmt"
	"visibility"
)

func consumeExported(v *visibility.Visibility) string {
	return v.GetExported()
}

func consumeDescribe(v *visibility.Visibility) string {
	return v.Describe()
}

func main() {
	v := visibility.NewVisibility()
	fmt.Println(consumeExported(v))
	fmt.Println(consumeDescribe(v))
}
