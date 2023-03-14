#![allow(clippy::missing_safety_doc)] // later

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::ffi::GoHandle;

pub mod ffi;

type TaskFuture = Mutex<Pin<Box<dyn Future<Output = ()>>>>;

pub struct Task<F: Future> {
    shared: Arc<SharedTask>,
    value: Arc<Mutex<Option<F::Output>>>,
}

impl<F: Future + 'static> Task<F> {
    pub fn spawn(fut: F) -> Self {
        let value = Arc::new(Mutex::new(None));
        let ret_value = value.clone();
        let pinned: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async move {
            *ret_value.lock().unwrap() = Some(fut.await);
        });
        let shared = Arc::new(SharedTask {
            handle: Mutex::new(GoHandle::nil()),
            fut: Mutex::new(pinned),
        });
        let handle = ffi::spawn_task(Arc::clone(&shared));
        *shared.handle.lock().unwrap() = handle;
        Self { shared, value }
    }
}

pub fn block_on<F: Future>(task: Task<F>) -> F::Output {
    let handle = *task.shared.handle.lock().unwrap();
    ffi::block_on(handle);
    task.value
        .lock()
        .unwrap()
        .take()
        .expect("future finished evaluating and should have returned")
}

pub struct SharedTask {
    handle: Mutex<ffi::GoHandle>,
    fut: TaskFuture,
}

enum MyWaker {}

impl MyWaker {
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
        let handle = *task.handle.lock().unwrap();
        ffi::wake_task(handle);
    }

    unsafe fn drop(data: *const ()) {
        Arc::decrement_strong_count(data);
    }
}
