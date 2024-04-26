use core::{
    future::Future,
    mem::{transmute, MaybeUninit},
    pin::Pin,
    sync::atomic::AtomicUsize,
};

use crate::ffi;

pub(crate) struct SharedTask<T, F: Future<Output = T>> {
    pub(crate) fut: Pin<*mut F>,
    pub(crate) result: MaybeUninit<T>,
    pub(crate) raw_handle: AtomicUsize,
}

pub fn block_on<T, F: Future<Output = T>>(mut fut: F) -> T {
    let pinned: Pin<&mut F> = unsafe { Pin::new_unchecked(&mut fut) };
    let pinned: Pin<*mut F> = unsafe { transmute(pinned) };
    let mut task = SharedTask {
        fut: pinned,
        result: MaybeUninit::uninit(),
        raw_handle: AtomicUsize::new(0),
    };

    let handle = unsafe { ffi::spawn_task(&mut task as *mut _) };

    ffi::block_on(&handle);

    unsafe { task.result.assume_init() }
}

#[cfg(test)]
mod tests {
    use core::task::{Context, Poll};

    use super::*;

    #[test]
    fn trivial_block_on_async_fn() {
        let () = block_on(async {});
    }

    #[test]
    fn trivial_block_on_empty_fut() {
        #[derive(Debug)]
        struct MyUselessFuture {
            has_run: bool,
        }
        impl Future for MyUselessFuture {
            type Output = ();

            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                dbg!(&self);
                assert!(!self.has_run);
                self.get_mut().has_run = true;
                Poll::Ready(())
            }
        }

        let fut = MyUselessFuture { has_run: true };
        dbg!(&fut);
        let () = block_on(fut);
    }
}
