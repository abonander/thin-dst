//! Atomic cell for Thin DSTs.

use super::{Invariant, ThinDst, WithVtable};

use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

pub struct AtomicDst<T: ?Sized> {
    ptr: AtomicPtr<WithVtable>,
    _type: Invariant<T>,
}

impl<T: ?Sized> AtomicDst<T> {
    pub fn new(opt: Option<ThinDst<T>>) -> Self {
        AtomicDst {
            ptr: AtomicPtr::new(opt_to_ptr(opt)),
            _type: Default::default(),
        }
    }

    pub fn swap_opt(&self, opt: Option<ThinDst<T>>, order: Ordering) -> Option<ThinDst<T>> {
        ThinDst::from_nullable_ptr(self.ptr.swap(opt_to_ptr(opt), order))
    }

    pub fn swap(&self, val: ThinDst<T>, order: Ordering) -> Option<ThinDst<T>> {
        self.swap_opt(Some(val), order)
    }

    pub fn take(&self, order: Ordering) -> Option<ThinDst<T>> {
        self.swap_opt(None, order)
    }
}

impl<T: ?Sized> Default for AtomicDst<T> {
    fn default() -> Self {
        Self::new(None)
    }
}

fn opt_to_ptr<T: ?Sized>(opt: Option<ThinDst<T>>) -> *mut WithVtable {
    opt.map_or_else(ptr::null_mut, ThinDst::into_ptr)
}

#[test]
fn test_atomic_basic() {
    let atomic = AtomicDst::new(None);
    atomic.swap(thin_dst!("Hello, world!" => ToString), Ordering::Relaxed);
    let dst = atomic.take(Ordering::Relaxed).unwrap();
    assert_eq!("Hello, world!".to_string(), dst.to_string());
}