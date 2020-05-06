use std::{
    fmt::{Debug, Error, Formatter},
    mem, ptr,
};

// This type replaces Rc<RefCell<T>> for Kal reference types (Lists, Objects, Strings, Closures)
// Unlike RefCell it does NOT provide interior mutability! You must have an &mut Ref<T> in
// order to get a Mut<T>. This means it should be impossible to create reference
// cycles among other things.

// It acts like Rc, and allows aliasing via clone()
// If you have a &mut Ref, you can ask to get a Mut to the value. This will not succeed,
// if there are other Kcs to this value.

// There cannot be more than one Mut given out by this type.
// We only give a Mut when ref_count == 1. Therefore, we will not return a Mut
// if there are any other Kcs to this value before calling borrow_mut.
// And, because we require a mutable reference, clone() cannot be called on this Ref until the mutable
// ref has been released. Rust's borrow checker will prevent it, because clone() would require taking
// another reference.

////////////////////////////////////
//   Public API
////////////////////////////////////
pub struct Ref<T> {
    // Pointer is always valid and non-null
    //
    // 1. We create it from a Box, which is always non null.
    // 2. We never re-assign the pointer.
    // 3. From 1. and 2., the pointer is always non-null.
    //
    // 4. When we hand out copies of the pointer (via clone()), we increment the ref-count.
    // 5. When we drop a copy of the pointer, we decrease the ref-count.
    // 6. From 3. and 4., the ref-count is always the same as the number of copies out there.
    // 7. We only drop the KcInner when the ref-count is 0.
    // 8. From 6. and 7., no pointers are ever dangling.
    //
    // 9. In order to borrow, there must be exactly one copy of the pointer.
    // 10. After borrowing, reading the value from the original copy of the pointer is now prevented.
    // 11. The value can only be read from the new Mut copy of the pointer.
    ptr: *mut KcInner<T>,
}
pub struct Mut<T> {
    // Pointer is always valid and non-null
    //
    // 1. We get this pointer from Ref<T>, which we assume is always valid an non-null.
    // 2. We never reassign the pointer.
    // 3. From 1. and 2., the pointer is always non-null.
    //
    // 4. When the Mut is created, the borrow sentinel is set.
    // 5. We never copy the pointer while the borrow sentinel is set.
    ptr: *mut KcInner<T>,
}

impl<T> Ref<T> {
    pub fn new(value: T) -> Self {
        Ref {
            ptr: Box::into_raw(Box::new(KcInner {
                ref_count: 1,
                value,
            })),
        }
    }

    pub fn try_clone(&self) -> Option<Self> {
        // allowed if there are other Ref but not if there is a Mut
        if self.is_borrowed() {
            None
        } else {
            self.inc_ref_count();
            Some(Self { ptr: self.ptr })
        }
    }

    pub fn try_get(&self) -> Option<&T> {
        // allowed if there are other Ref but not if there is a Mut
        if self.is_borrowed() {
            None
        } else {
            Some(unsafe { &(*self.ptr).value })
        }
    }

    // Get a mutable reference to the inner value. This will be released according to Rust's rules.
    pub fn try_get_mut(&mut self) -> Option<&mut T> {
        // Only allowed if there are no other Ref
        if self.ref_count() > 1 {
            None
        } else {
            Some(unsafe { &mut (*self.ptr).value })
        }
    }

    // Get a Mut to the inner value. This will be released dynamically, and shares ownership of the inner value.
    pub fn try_borrow(&mut self) -> Option<Mut<T>> {
        // Only allowed if there are no other Ref
        if self.ref_count() > 1 {
            None
        } else {
            self.borrow_inner();
            Some(Mut { ptr: self.ptr })
        }
    }

    pub fn try_into_inner(self) -> Result<T, Self> {
        // Only allowed if there are no other Ref
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

impl<T> Mut<T> {
    pub fn get(&self) -> &T {
        unsafe { &(*self.ptr).value }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut (*self.ptr).value }
    }

    // Turn this Mut into a Ref, allowing the original Ref to be used again.
    pub fn into_ref(mut self) -> Ref<T> {
        // Set ref_count back to 1
        self.unborrow_inner();

        // Create the new Ref. There are now two Ref including the original, so we
        // increment the ref_count.
        let r = Ref { ptr: self.ptr };
        r.inc_ref_count();

        // don't run drop
        std::mem::forget(self);

        r
    }
}

////////////////////////////////////
//   Public impls
////////////////////////////////////
impl<T: PartialEq> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        let this = self.try_get().expect(
            "Implementation error - couldn't get value in eq() on Ref, it is mutably borrowed.",
        );
        let other = other.try_get().expect(
            "Implementation error - couldn't get value in eq() on Ref, it is mutably borrowed.",
        );
        this.eq(other)
    }
}
impl<T: PartialEq> PartialEq for Mut<T> {
    fn eq(&self, other: &Self) -> bool {
        let this = self.get();
        let other = other.get();
        this.eq(other)
    }
}

impl<T: Eq> Eq for Ref<T> {}
impl<T: Eq> Eq for Mut<T> {}

impl<T: PartialOrd> PartialOrd for Ref<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let this = self.try_get().expect(
            "Implementation error - couldn't get value in partial_cmp() on Ref, it is mutably borrowed.",
        );
        let other = other.try_get().expect(
            "Implementation error - couldn't get value in partial_cmp() on Ref, it is mutably borrowed.",
        );
        this.partial_cmp(other)
    }
}
impl<T: PartialOrd> PartialOrd for Mut<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let this = self.get();
        let other = other.get();
        this.partial_cmp(other)
    }
}

impl<T: Debug> Debug for Ref<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let this = self.try_get().expect(
            "Implementation error - couldn't get value in fmt() on Ref, it is mutably borrowed.",
        );
        this.fmt(fmt)
    }
}
impl<T: Debug> Debug for Mut<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let this = self.get();
        this.fmt(fmt)
    }
}

////////////////////////////////////
///   Private implementation details
////////////////////////////////////

// We just do interior mutability here because we are using unsafe anyway.
impl<T> Ref<T> {
    fn ref_count(&self) -> usize {
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
    fn borrow_inner(&self) {
        unsafe {
            (*self.ptr).ref_count = usize::MAX;
        }
    }
    fn is_borrowed(&self) -> bool {
        unsafe { (*self.ptr).ref_count == usize::MAX }
    }
}
impl<T> Mut<T> {
    fn unborrow_inner(&self) {
        unsafe {
            (*self.ptr).ref_count = 1;
        }
    }
}

impl<T> Drop for Ref<T> {
    fn drop(&mut self) {
        self.dec_ref_count();
        if self.ref_count() == 0 {
            unsafe { ptr::drop_in_place(self.ptr) }
        }
    }
}
impl<T> Drop for Mut<T> {
    fn drop(&mut self) {
        self.unborrow_inner();
    }
}

struct KcInner<T> {
    ref_count: usize,
    value: T,
}

