package main

// void rust_hello_world();
import (
	"C"
)
import "fmt"

//export HelloGo
func HelloGo() {
	fmt.Println("Hello from Go runtime")
	C.rust_hello_world()
}

func main() {}
