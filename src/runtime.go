package main

/*
#include <stdint.h>

// Argument is a Rust gotime::TaskFuture without the shared pointer
// Return is a bool indicating if the task finished
char gotime_poll_task(void*);
*/
import "C"
import (
	"runtime"
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

type AllocationInfo struct {
	pinner     runtime.Pinner
	allocation []C.char
}

//export gotime_allocate
func gotime_allocate(size C.size_t, align C.size_t) (C.uintptr_t, C.uintptr_t) {
	var allocationSize = size + align
	// FIXME: This size is pessimistic, it should avoid creating offsets if the
	// alignment would be satisfied by default (unless this doesn't matter for perf)
	var allocation = make([]C.char, allocationSize)

	var info = AllocationInfo{
		allocation: allocation,
	}
	info.pinner.Pin(&allocation)

	var allocationPointer = C.uintptr_t(uintptr(unsafe.Pointer(&allocation)))
	// https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
	var alignedPointer = (allocationPointer + (align - 1)) & -align

	return C.uintptr_t(cgo.NewHandle(info)), C.uintptr_t(alignedPointer)
}

//export gotime_clone_allocation
func gotime_clone_allocation(handle C.uintptr_t) C.uintptr_t {
	var info = cgo.Handle(handle).Value().(AllocationInfo)
	var newPinner runtime.Pinner

	newPinner.Pin(&info.allocation)

	var newInfo = AllocationInfo{
		pinner:     newPinner,
		allocation: info.allocation,
	}
	return C.uintptr_t(cgo.NewHandle(newInfo))
}

//export gotime_free
func gotime_free(handle C.uintptr_t) {
	var h = cgo.Handle(handle)
	var info = h.Value().(AllocationInfo)
	info.pinner.Unpin()
	h.Delete()
}

func main() {}
