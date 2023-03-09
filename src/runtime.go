package main

/*
#include <stdint.h>
typedef struct {
	uintptr_t handle;
} Runtime;

typedef struct {
	// Rust: &dyn Future
	void *future;
	// Go: chan C.Task
	uintptr_t handle;
} Task;

void gotime_process_task(Task*);
*/
import "C"
import (
	"fmt"
	"runtime"
	"runtime/cgo"
)

type task_chan chan *C.Task

//export gotime_start_runtime
func gotime_start_runtime() C.Runtime {
	fmt.Println("Go: Runtime started")
	ch := make(task_chan)
	go func() {
		for task := range ch {
			fmt.Println("Go: Got Task: ", task)
			C.gotime_process_task(task)
			fmt.Println("Go: Finished Task ", task)
		}
	}()
	return C.Runtime{
		handle: C.uintptr_t(cgo.NewHandle(ch)),
	}
}

//export gotime_submit_task
func gotime_submit_task(task *C.Task) {
	fmt.Println("Go: Task submitted")
	ch := retrieve_runtime_channel(C.Runtime{task.handle})
	ch <- task
}

//export gotime_close_runtime
func gotime_close_runtime(runtime C.Runtime) {
	fmt.Println("Go: Runtime closed")
	close(retrieve_runtime_channel(runtime))
	cgo.Handle(runtime.handle).Delete()
}

//export gotime_poll_futures
func gotime_poll_futures() {
	// always yield
	for {
		runtime.Gosched()
	}
}

func retrieve_runtime_channel(runtime C.Runtime) task_chan {
	return cgo.Handle(runtime.handle).Value().(task_chan)
}

func main() {}
