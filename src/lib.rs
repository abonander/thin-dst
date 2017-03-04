use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::{mem, ptr};

#[macro_export]
macro_rules! thin_dst (
    ($expr:expr => $dst:ty) => (
        {
            let boxed: Box<$crate::ThinPrimer<$dst>> = Box::new($crate::ThinPrimer::new($expr));
            boxed.into_thin()
        }
    )
);

#[derive(Debug)]
struct FatPtr {
    data: *mut (),
    vtable: *const (),
}

impl FatPtr {
    fn from_box<T: ?Sized>(obj: Box<T>) -> Self {
        assert_eq!(mem::size_of::<*mut T>(), mem::size_of::<FatPtr>());

        let obj_ptr = Box::into_raw(obj);

        unsafe {
            ptr::read((&obj_ptr) as *const *mut T as *const FatPtr)
        }
    }

    fn to_ptr<T: ?Sized>(&self) -> *mut T {
        assert_eq!(mem::size_of::<*mut T>(), mem::size_of::<FatPtr>());
        unsafe {
            let obj_ptr = self as *const FatPtr as *const *mut T;
            *obj_ptr
        }
    }
}

/// Implementation detail
// Prevent reordering of fields
#[repr(C)]
#[doc(hidden)]
pub struct ThinPrimer<T: ?Sized> {
    vtable: *const (),
    val: T,
}

impl<T: ?Sized> ThinPrimer<T> {
    pub fn new(val: T) -> ThinPrimer<T> where T: Sized {
        ThinPrimer {
            vtable: ptr::null(),
            val: val,
        }
    }

    pub fn into_thin(self: Box<Self>) -> ThinDst<T> {
        let fat_ptr = FatPtr::from_box(self);

        let obj_ptr = fat_ptr.data as *mut WithVtable;

        unsafe {
            (*obj_ptr).vtable = fat_ptr.vtable;
        }

        ThinDst {
            ptr: obj_ptr,
            _trait: PhantomData,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct WithVtable {
    vtable: *const (),
    data: i32,
}

impl WithVtable {
    unsafe fn fat_ptr(self_: *mut WithVtable) -> FatPtr {
        let fat_ptr = FatPtr {
            data: self_ as *mut (),
            vtable: (*self_).vtable
        };

        fat_ptr
    }
}

pub struct ThinDst<T: ?Sized> {
    ptr: *mut WithVtable,
    _trait: PhantomData<T>,
}

impl<T: ?Sized> ThinDst<T> {
    unsafe fn primer_ptr(&self) -> *mut ThinPrimer<T> {
        WithVtable::fat_ptr(self.ptr).to_ptr()
    }
}

impl<T: ?Sized> Deref for ThinDst<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &(*self.primer_ptr()).val
        }
    }
}

impl<T: ?Sized> DerefMut for ThinDst<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut (*self.primer_ptr()).val
        }
    }
}

impl<T: ?Sized> Drop for ThinDst<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.primer_ptr()));
        }
    }
}

#[test]
fn test_basic() {
    let display = thin_dst!("Hello, world!" => ToString);
    assert_eq!(display.to_string(), "Hello, world!");
    mem::forget(display);
}