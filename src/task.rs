use alloc::{boxed::Box, sync::Arc};
use core::{cell::UnsafeCell, future::Future, pin::Pin};

use crate::ffi::{self, GoHandle};

type TaskFuture = UnsafeCell<Pin<Box<dyn Future<Output = ()>>>>;

pub(crate) struct SharedTask {
    pub(crate) handle: UnsafeCell<ffi::GoHandle>,
    pub(crate) fut: TaskFuture,
}

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
