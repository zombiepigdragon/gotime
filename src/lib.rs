#![allow(clippy::missing_safety_doc)] // later

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

pub mod ffi;

type TaskFuture = Box<Mutex<Pin<Box<dyn Future<Output = ()>>>>>;

pub struct Runtime(ffi::generated::Runtime);

impl Runtime {
    pub fn new() -> Self {
        unsafe { Runtime(ffi::generated::gotime_start_runtime()) }
    }

    // FIXME: This might need to be &mut self, not sure how concurrent Go is here
    pub fn spawn<F>(&self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let fut = Box::pin(fut);
        let fut: TaskFuture = Box::new(Mutex::new(fut));
        let task = Arc::new(ffi::generated::Task {
            future: Box::into_raw(fut).cast(),
            handle: self.0.handle,
        });
        unsafe {
            ffi::generated::gotime_submit_task(Arc::into_raw(task).cast_mut());
        }
    }

    // FIXME: This can't return because the Go side loops forever.
    // it does that because we otherwise can't wait for the futures to resolve
    pub fn block_on_remaining(self) {
        unsafe {
            ffi::generated::gotime_poll_futures();
        }
        drop(self)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe { ffi::generated::gotime_close_runtime(self.0) };
    }
}

mod waker {
    use std::{
        sync::Arc,
        task::{RawWaker, RawWakerVTable, Waker},
    };

    static V_TABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    pub(super) fn wrap_arc_fut(fut: &Arc<crate::ffi::generated::Task>) -> Waker {
        let ptr = Arc::into_raw(Arc::clone(fut));

        let raw_waker = RawWaker::new(ptr.cast(), &V_TABLE);
        unsafe { Waker::from_raw(raw_waker) }
    }

    unsafe fn clone(data: *const ()) -> RawWaker {
        Arc::increment_strong_count(data);
        RawWaker::new(data, &V_TABLE)
    }

    unsafe fn wake(data: *const ()) {
        wake_by_ref(data);
        drop(data);
    }

    unsafe fn wake_by_ref(data: *const ()) {
        super::ffi::generated::gotime_submit_task(data.cast_mut().cast());
    }

    unsafe fn drop(data: *const ()) {
        let _ = Arc::from_raw(data.cast::<crate::ffi::generated::Task>());
    }
}
