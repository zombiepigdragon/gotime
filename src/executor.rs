use core::task::{RawWaker, RawWakerVTable, Waker};

use crate::ffi::{self, GoHandle};

pub(crate) enum MyWaker {}

impl MyWaker {
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new(raw_handle: usize) -> Waker {
        println!("new waker");
        let raw_waker = RawWaker::new(raw_handle as *const (), Self::vtable());
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
        println!("waker clone");
        RawWaker::new(data, Self::vtable())
    }

    unsafe fn wake(data: *const ()) {
        println!("waker wake");
        Self::wake_by_ref(data);
        Self::drop(data)
    }

    unsafe fn wake_by_ref(data: *const ()) {
        println!("waker wake by ref");
        let raw_handle = data as usize;
        debug_assert_ne!(raw_handle, 0, "handle is nil at wake time");
        let handle = &GoHandle::from_raw(raw_handle);
        println!("waker waking with FFI");
        ffi::wake_task(handle);
    }

    unsafe fn drop(_: *const ()) {
        println!("waker drop");
        // we don't care about this; the waker owns nothing
    }
}
