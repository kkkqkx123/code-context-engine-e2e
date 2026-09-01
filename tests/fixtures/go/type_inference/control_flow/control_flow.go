package main

import (
	"errors"
	"fmt"
	"strconv"
)

func handleError(err error) string {
	if err != nil {
		return err.Error()
	}
	return "no error"
}

func divide(a, b int) (int, error) {
	if b == 0 {
		return 0, errors.New("division by zero")
	}
	return a / b, nil
}

func parseAndProcess(s string) (int, error) {
	n, err := strconv.Atoi(s)
	if err != nil {
		return 0, fmt.Errorf("parse error: %w", err)
	}
	if n < 0 {
		return 0, errors.New("negative number")
	}
	return n * 2, nil
}

func processTwoErrors(a, b string) (int, error) {
	n1, err := strconv.Atoi(a)
	if err != nil {
		return 0, err
	}
	n2, err := strconv.Atoi(b)
	if err != nil {
		return 0, err
	}
	return n1 + n2, nil
}

func main() {
	err := errors.New("test error")
	fmt.Println(handleError(err))
	fmt.Println(handleError(nil))

	result, err := divide(10, 3)
	if err != nil {
		fmt.Println("error:", err)
	} else {
		fmt.Println("result:", result)
	}

	parsed, err := parseAndProcess("42")
	if err != nil {
		fmt.Println("error:", err)
	} else {
		fmt.Println("parsed:", parsed)
	}
}
