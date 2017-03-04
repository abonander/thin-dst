use std::marker::{PhantomData, Unsize};
use std::ops::{Deref, DerefMut};
use std::{mem, ptr};

struct TraitObj {
    data: *mut (),
    vtable: *const (),
}

impl TraitObj {
    fn from_box<T: ?Sized>(obj: Box<T>) -> Self {
        assert_eq!(mem::size_of::<Box<T>>(), mem::size_of::<TraitObj>());

        let trait_obj = unsafe {
            let obj_ptr = &&obj as *const Box<T> as *const TraitObj;
            *obj_ptr
        };

        mem::forget(obj);

        trait_obj
    }
}

// Prevent reordering of fields
#[repr(C)]
struct ThinPrimer<T: ?Sized> {
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

    pub fn into_thin(mut self: Box<Self>) -> ThinTraitObj<T> {
        let trait_obj = TraitObj::from_box(self);

        let obj_ptr = trait_obj.data as *mut WithVtable;

        unsafe {
            *obj_ptr.vtable = trait_obj.vtable;
        }

        ThinTraitObj {
            ptr: obj_ptr,
            _trait: PhantomData,
        }
    }
}

#[repr(C)]
struct WithVtable {
    vtable: *const (),
    data: (),
}

struct ThinTraitObj<T: ?Sized> {
    ptr: *mut WithVtable,
    _trait: PhantomData<T>,
}

impl Deref for ThinTraitObj {
    type Target = T;

    fn deref(&self) -> &T {

    }
}