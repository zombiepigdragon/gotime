package main

/*
#include <stdint.h>

// A function that can be called to poll a future.
// In Rust, this is the instantiated form of a generic wrapper around poll.
// Argument is a Rust SharedTask.
// Return is non-zero if Poll::Ready.
typedef uint8_t (*poll_callback)(void*);
static uint8_t invoke_poll_callback(poll_callback f, void *shared_task) {return f(shared_task);}

typedef void (*drop_callback)(void*);
// FIXME: uintptr_t isn't the right type, but Go crashes when passing a void* and using unsafe.Pointer everywhere
static void invoke_drop(drop_callback f, uintptr_t value) {f((void*)value);}
*/
import "C"
import (
	"runtime"
	"runtime/cgo"
	"sync/atomic"
	"unsafe"
)

type Task struct {
	poll_func   C.poll_callback
	shared_task *C.void
	waker       chan struct{}
	finished    chan struct{}
}

//export gotime_spawn_task
func gotime_spawn_task(poll_func C.poll_callback, shared_task *C.void) C.uintptr_t {
	var task = Task{
		poll_func:   poll_func,
		shared_task: shared_task,
		waker:       make(chan struct{}),
		finished:    make(chan struct{}),
	}
	go func() {
		println("go: spawned task")
		for C.invoke_poll_callback(task.poll_func, unsafe.Pointer(shared_task)) == 0 {
			println("go: polled task")
			// wait for an item to arrive
			_ = <-task.waker
			println("go: waiting on task")
		}
		println("go: finished task")
		task.finished <- struct{}{}

	}()
	var handle = C.uintptr_t(cgo.NewHandle(task))
	println("go: handle is", handle)
	return handle
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
	refCount   *atomic.Uint64
	aligned    uintptr
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
		refCount:   new(atomic.Uint64),
	}
	info.pinner.Pin(&allocation)
	info.refCount.Add(1)

	var allocationPointer = C.uintptr_t(uintptr(unsafe.Pointer(&allocation)))
	// https://en.wikipedia.org/wiki/Data_structure_alignment#Computing_padding
	var alignedPointer = (allocationPointer + (align - 1)) & -align
	info.aligned = uintptr(alignedPointer)

	return C.uintptr_t(cgo.NewHandle(info)), C.uintptr_t(alignedPointer)
}

//export gotime_clone_allocation
func gotime_clone_allocation(handle C.uintptr_t) C.uintptr_t {
	var info = cgo.Handle(handle).Value().(AllocationInfo)
	var newPinner runtime.Pinner

	newPinner.Pin(&info.allocation)
	info.refCount.Add(1)

	var newInfo = AllocationInfo{
		pinner:     newPinner,
		refCount:   info.refCount,
		allocation: info.allocation,
		aligned:    info.aligned,
	}
	return C.uintptr_t(cgo.NewHandle(newInfo))
}

//export gotime_free
func gotime_free(handle C.uintptr_t, on_drop C.drop_callback) {
	var h = cgo.Handle(handle)
	var info = h.Value().(AllocationInfo)

	var minus_one = ^uint64(0) // documented workaround for no `.Sub`
	var refCount = info.refCount.Add(minus_one)
	if refCount == 0 {
		C.invoke_drop(on_drop, C.uintptr_t(info.aligned))
	}

	// release this hold on the memory
	info.pinner.Unpin()
	h.Delete()
}

func main() {}
