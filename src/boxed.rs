use core::{marker::PhantomData, ops::Deref, ptr::NonNull};

use crate::ffi::{self, GoHandle};

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
        ffi::free::<T>(core::mem::replace(&mut self.handle, GoHandle::nil()));
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

    #[test]
    fn go_box_drop() {
        use core::sync::atomic::{AtomicBool, Ordering::SeqCst};

        struct NeedsDrop<'a> {
            dropped: &'a AtomicBool,
        }
        impl Drop for NeedsDrop<'_> {
            fn drop(&mut self) {
                self.dropped.store(true, SeqCst)
            }
        }

        let was_dropped = AtomicBool::new(false);
        let needs_drop = NeedsDrop {
            dropped: &was_dropped,
        };

        let boxed = GoBox::new(needs_drop);
        assert!(!was_dropped.load(SeqCst), "dropped at boxing");
        let boxed2 = boxed.clone();
        assert!(!was_dropped.load(SeqCst), "dropped at clone");
        drop(boxed2);
        assert!(!was_dropped.load(SeqCst), "dropped on first free");
        drop(boxed);
        assert!(was_dropped.load(SeqCst), "not dropped");
    }
}
