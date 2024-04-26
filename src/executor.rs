use core::task::{RawWaker, RawWakerVTable, Waker};

use alloc::sync::Arc;

use crate::{ffi, task::SharedTask};

pub(crate) enum MyWaker {}

impl MyWaker {
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new(task: Arc<SharedTask>) -> Waker {
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
