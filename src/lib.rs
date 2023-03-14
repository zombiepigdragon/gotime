#![allow(clippy::missing_safety_doc)] // later

use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
};

use crate::ffi::GoHandle;

pub mod ffi;

type TaskFuture = Mutex<Pin<Box<dyn Future<Output = ()>>>>;

pub struct Task<F: Future> {
    shared: Arc<SharedTask>,
    _marker: PhantomData<F>,
}

impl<F: Future<Output = ()> + 'static> Task<F> {
    pub fn spawn(fut: F) -> Self {
        let pinned: Pin<Box<dyn Future<Output = ()>>> = Box::pin(fut);
        let shared = Arc::new(SharedTask {
            handle: Mutex::new(GoHandle::nil()),
            fut: Mutex::new(pinned),
        });
        let handle = ffi::spawn_task(Arc::clone(&shared));
        *shared.handle.lock().unwrap() = handle;
        Self {
            shared,
            _marker: PhantomData,
        }
    }
}

pub fn block_on<F: Future>(task: Task<F>) {
    let handle = *task.shared.handle.lock().unwrap();
    ffi::block_on(handle);
    todo!("retrieve the output value")
}

pub struct SharedTask {
    handle: Mutex<ffi::GoHandle>,
    fut: TaskFuture,
}

mod waker {
    use std::{
        sync::Arc,
        task::{RawWaker, RawWakerVTable, Waker},
    };

    use crate::SharedTask;

    // type of the data passed to methods is SharedTask created from Arc
    static V_TABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    pub(super) fn new(task: Arc<SharedTask>) -> Waker {
        let raw_waker = RawWaker::new(Arc::into_raw(task).cast(), &V_TABLE);
        unsafe { Waker::from_raw(raw_waker) }
    }

    unsafe fn clone(data: *const ()) -> RawWaker {
        Arc::increment_strong_count(data);
        RawWaker::new(data, &V_TABLE)
    }

    unsafe fn wake(data: *const ()) {
        wake_by_ref(data);
        super::waker::drop(data)
    }

    unsafe fn wake_by_ref(data: *const ()) {
        let task: &SharedTask = data.cast::<SharedTask>().as_ref().unwrap();
        let handle = *task.handle.lock().unwrap();
        super::ffi::wake_task(handle);
    }

    unsafe fn drop(data: *const ()) {
        Arc::decrement_strong_count(data);
    }
}
