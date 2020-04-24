use std::{
    fmt::{Debug, Error, Formatter},
    mem,
    ops::Deref,
    ptr,
};

// This type replaces Rc<RefCell<T>> for Kal reference types (Lists, Objects, Strings, Closures)
// Unlike RefCell it does NOT provide interior mutability! You must have a mutable KalRef in
// order to get a mutable ref to the value. This means it should be impossible to create reference
// cycles among other things.

// It acts like Rc, and allows aliasing via clone()
// If you have a mutable KalRef, you can ask to get a mutable ref to the value. This will not succeed,
// if there are other KalRefs to this value.

// There cannot be more than one mutable reference given out by this type.
// We only give a mutable reference when ref_count == 1. Therefore, we will not return a mutable ref
// if there are any other KalRefs to this value before calling borrow_mut.
// And, because we require a mutable reference, clone() cannot be called on this KalRef until the mutable
// ref has been released. Rust's borrow checker will prevent it, because clone() would require taking
// another reference.

////////////////////////////////////
///   Public API
////////////////////////////////////
pub struct KalRef<T> {
    // Pointer is always valid and non-null
    //
    // 1. We create it from a Box, which is always non null.
    // 2. We never re-assign the pointer.
    // 3. From 1. and 2., the pointer is always non-null.
    //
    // 4. When we hand out copies of the pointer (via clone()), we increment the ref-count.
    // 5. When we drop a copy of the pointer, we decrease the ref-count.
    // 6. From 3. and 4., the ref-count is always the same as the number of copies out there.
    // 7. We only drop the KalRefInner when the ref-count is 0.
    // 8. From 6. and 7., no pointers are ever dangling.
    ptr: *mut KalRefInner<T>,
}

impl<T> KalRef<T> {
    pub fn new(value: T) -> Self {
        KalRef {
            ptr: Box::into_raw(Box::new(KalRefInner {
                ref_count: 1,
                value,
            })),
        }
    }

    pub fn borrow_mut(&mut self) -> Option<&mut T> {
        if self.ref_count() > 1 {
            None
        } else {
            Some(unsafe { &mut (*self.ptr).value })
        }
    }

    pub fn try_into_inner(self) -> Result<T, Self> {
        if self.ref_count() > 1 {
            Err(self)
        } else {
            let inner = unsafe { ptr::read(self.ptr) };
            let value = inner.value;
            mem::forget(self);
            Ok(value)
        }
    }
}

////////////////////////////////////
///   Public impls
////////////////////////////////////
impl<T> Clone for KalRef<T> {
    fn clone(&self) -> Self {
        self.inc_ref_count();
        Self { ptr: self.ptr }
    }
}

impl<T> Deref for KalRef<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { &(*self.ptr).value }
    }
}

impl<T: PartialEq> PartialEq for KalRef<T> {
    fn eq(&self, other: &Self) -> bool {
        (&**self).eq(other)
    }
}

impl<T: Eq> Eq for KalRef<T> {}

impl<T: PartialOrd> PartialOrd for KalRef<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (&**self).partial_cmp(other)
    }
}

impl<T: Debug> Debug for KalRef<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        (&**self).fmt(fmt)
    }
}

////////////////////////////////////
///   Private implementation details
////////////////////////////////////
impl<T> KalRef<T> {
    fn ref_count(&self) -> u64 {
        unsafe { (*self.ptr).ref_count }
    }
    fn inc_ref_count(&self) {
        unsafe {
            (*self.ptr).ref_count += 1;
        }
    }
    fn dec_ref_count(&self) {
        unsafe {
            (*self.ptr).ref_count -= 1;
        }
    }
}

impl<T> Drop for KalRef<T> {
    fn drop(&mut self) {
        self.dec_ref_count();
        if self.ref_count() == 0 {
            unsafe { ptr::drop_in_place(self.ptr) }
        }
    }
}

struct KalRefInner<T> {
    ref_count: u64,
    value: T,
}