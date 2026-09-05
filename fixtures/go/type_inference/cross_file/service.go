package main

import (
	"fmt"
	"models"
)

func RenderGreeting(user *models.User) string {
	return user.Greet()
}

func main() {
	user := models.LoadUser("Alice")
	fmt.Println(RenderGreeting(user))
	fmt.Println(user.Greet())
}
