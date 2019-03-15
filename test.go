package main

import (
	"fmt"
	
	"time"
)

func a (){
	for {
		fmt.Println("Hallo ... a")
		time.Sleep( 2 * time.Second )
	}
}


func b (){
	for {
		fmt.Println("Hallo ... b")
		time.Sleep( 2 * time.Second )
	}
}


func main() {
	//ci := make(chan int) 
	

	go a()
	go b()
	for {
		fmt.Println(".... waiting ")
		time.Sleep( 2 * time.Second )
	}
}
