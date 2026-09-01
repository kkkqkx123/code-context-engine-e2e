package main

import "fmt"

type DataProcessor struct {
	items []string
}

func (dp *DataProcessor) AddItem(item string) {
	dp.items = append(dp.items, item)
}

func (dp *DataProcessor) GetItems() []string {
	result := make([]string, len(dp.items))
	copy(result, dp.items)
	return result
}

func (dp *DataProcessor) Count() int {
	return len(dp.items)
}

func main() {
	dp := &DataProcessor{}
	dp.AddItem("hello")
	dp.AddItem("world")
	fmt.Println(dp.Count())
}
