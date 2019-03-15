package main

import (
	"fmt"
	"time"
)

func send1(ch chan int) {
	i := 0
	for {
		fmt.Println("send1 --> ", i)
		ch <- i
		time.Sleep(5 * time.Second)
		i++
	}
}

func send2(ch chan int) {
	i := 0
	for {
		fmt.Println("send2 --> ", i)
		ch <- i
		time.Sleep(5 * time.Second)
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

	go send1(ch)
	go send2(ch)
	go rec(ch)
	for {
		fmt.Println(".... waiting ")
		time.Sleep(20 * time.Second)
	}
}
