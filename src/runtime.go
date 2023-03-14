package main

/*
#include <stdint.h>

// Argument is a Rust gotime::TaskFuture without the shared pointer
// Return is a bool indicating if the task finished
char gotime_poll_task(void*);
*/
import "C"
import (
	"runtime/cgo"
	"unsafe"
)

type Task struct {
	future   *C.void
	waker    chan struct{}
	finished chan struct{}
}

//export gotime_spawn_task
func gotime_spawn_task(future *C.void) C.uintptr_t {
	var task = Task{
		future:   future,
		waker:    make(chan struct{}),
		finished: make(chan struct{}),
	}
	go func() {
		for {
			if C.gotime_poll_task(unsafe.Pointer(task.future)) == 0 {
				task.finished <- struct{}{}
				break
			}
			// wait for an item to arrive
			_ = <-task.waker
		}
	}()
	return C.uintptr_t(cgo.NewHandle(task))
}

//export gotime_wake_task
func gotime_wake_task(handle C.uintptr_t) {
	var task = cgo.Handle(handle).Value().(Task)
	task.waker <- struct{}{}
}

//export gotime_block_on
func gotime_block_on(handle C.uintptr_t) {
	var task = cgo.Handle(handle).Value().(Task)
	_ = <-task.finished
}

func main() {}
