package main

/*
#include <stdint.h>
typedef struct {
	uintptr_t handle;
} Runtime;
*/
import "C"
import (
	"fmt"
	"runtime/cgo"
)

type task_chan chan int32

//export gotime_start_runtime
func gotime_start_runtime() C.Runtime {
	ch := make(task_chan)
	go func() {
		for task := range ch {
			fmt.Println("Go: Got \"Task\": ", task)
		}
	}()
	return C.Runtime{
		handle: C.uintptr_t(cgo.NewHandle(ch)),
	}
}

//export gotime_submit_task
func gotime_submit_task(runtime C.Runtime, task int32) {
	ch := retrieve_runtime_channel(runtime)
	ch <- task
}

//export gotime_close_runtime
func gotime_close_runtime(runtime C.Runtime) {
	cgo.Handle(runtime.handle).Delete()
}

func retrieve_runtime_channel(runtime C.Runtime) task_chan {
	return cgo.Handle(runtime.handle).Value().(task_chan)
}

func main() {}
