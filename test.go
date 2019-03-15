package main

import (
	"fmt"

	"time"
)

func send(ch chan int) {
	i := 0
	for {
		fmt.Println("send --> ", i)
		time.Sleep(5 * time.Second)
		ch <- i
		i++
	}
}

func rec(ch chan int) {
	for {
		i := <-ch
		fmt.Println("rec  --> ", i)
		fmt.Println("-----------------")
	}
}

func main() {
	ch := make(chan int)

	go send(ch)
	go rec(ch)
	for {
		fmt.Println(".... waiting ")
		time.Sleep(20 * time.Second)
	}
}
