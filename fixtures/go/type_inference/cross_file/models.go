package models

type User struct {
	Name string
	Age  int
}

func NewUser(name string, age int) *User {
	return &User{Name: name, Age: age}
}

func (u *User) Greet() string {
	return "Hello, " + u.Name + "!"
}

func LoadUser(name string) *User {
	return NewUser(name, 30)
}
