#![allow(clippy::missing_safety_doc)] // later
#![no_std]

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use core::{
    cell::UnsafeCell,
    future::Future,
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    ptr::NonNull,
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::ffi::GoHandle;

pub mod ffi;

type TaskFuture = UnsafeCell<Pin<Box<dyn Future<Output = ()>>>>;

pub struct Task<F: Future> {
    shared: Arc<SharedTask>,
    value: Arc<UnsafeCell<Option<F::Output>>>,
}

impl<F: Future + 'static> Task<F> {
    pub fn spawn(fut: F) -> Self {
        let value = Arc::new(UnsafeCell::new(None));
        let ret_value = value.clone();
        let pinned: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async move {
            unsafe {
                *ret_value.get() = Some(fut.await);
            }
        });
        let shared = Arc::new(SharedTask {
            handle: UnsafeCell::new(GoHandle::nil()),
            fut: UnsafeCell::new(pinned),
        });
        let handle = ffi::spawn_task(Arc::clone(&shared));
        unsafe {
            *shared.handle.get() = handle;
        }
        Self { shared, value }
    }
}

pub fn block_on<F: Future>(task: Task<F>) -> F::Output {
    let handle = unsafe { &*task.shared.handle.get() };
    ffi::block_on(handle);
    unsafe {
        (*task.value.get())
            .take()
            .expect("future finished evaluating and should have returned")
    }
}

pub struct SharedTask {
    handle: UnsafeCell<ffi::GoHandle>,
    fut: TaskFuture,
}

enum MyWaker {}

impl MyWaker {
    #[allow(clippy::new_ret_no_self)]
    fn new(task: Arc<SharedTask>) -> Waker {
        let raw_waker = RawWaker::new(Arc::into_raw(task).cast(), Self::vtable());
        unsafe { Waker::from_raw(raw_waker) }
    }

    fn vtable() -> &'static RawWakerVTable {
        static V_TABLE: RawWakerVTable = RawWakerVTable::new(
            MyWaker::clone,
            MyWaker::wake,
            MyWaker::wake_by_ref,
            MyWaker::drop,
        );
        &V_TABLE
    }

    unsafe fn clone(data: *const ()) -> RawWaker {
        Arc::increment_strong_count(data);
        RawWaker::new(data, Self::vtable())
    }

    unsafe fn wake(data: *const ()) {
        Self::wake_by_ref(data);
        Self::drop(data)
    }

    unsafe fn wake_by_ref(data: *const ()) {
        let task: &SharedTask = data.cast::<SharedTask>().as_ref().unwrap();
        let handle = &*task.handle.get();
        ffi::wake_task(handle);
    }

    unsafe fn drop(data: *const ()) {
        Arc::decrement_strong_count(data);
    }
}

/// Represents a memory allocation into Go.
pub struct GoBox<T> {
    ptr: NonNull<T>,
    handle: GoHandle,
    _marker: PhantomData<T>,
}

impl<T> GoBox<T> {
    pub fn new(value: T) -> Self {
        let (handle, uninit_ptr) = ffi::allocate::<T>();
        let place = unsafe { &mut *uninit_ptr };
        let ptr = place.write(value);
        let ptr = NonNull::from(ptr);
        Self {
            ptr,
            handle,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for GoBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Drop for GoBox<T> {
    fn drop(&mut self) {
        ffi::free(core::mem::replace(&mut self.handle, GoHandle::nil()));
    }
}

impl<T> Clone for GoBox<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            handle: ffi::clone_allocation(&self.handle),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_box_basic_functionality() {
        let value: i32 = 0xABCDEF0; // random value, somewhat unlikely to just appear
        let boxed = GoBox::new(value);
        assert_eq!(value, *boxed, "value not equal after boxing");
        let cloned = boxed.clone();
        assert_eq!(value, *boxed, "boxed value not equal after cloning");
        assert_eq!(*boxed, *cloned, "boxed value and cloned value not equal");
        drop(boxed);
        assert_eq!(value, *cloned, "cloned value not equal to value after drop");
        drop(cloned);
    }

    #[test]
    fn go_box_strange_align() {
        // type that is unlikely to have alignment met by coincidence
        #[repr(align(2048))]
        struct WeirdAlign {
            _storage: u8,
        }

        let boxed = GoBox::new(WeirdAlign { _storage: 0 });
        core::hint::black_box(&*boxed); // ensure box is actually allocated
    }

    #[test]
    fn go_box_zst() {
        let boxed = GoBox::new(());
        core::hint::black_box(&*boxed); // ensure box is actually allocated
    }
}
