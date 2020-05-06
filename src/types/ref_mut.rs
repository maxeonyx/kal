/*!
An alternative to Rc<RefCell<T>> for Kal reference types (Lists, Objects, Strings, Closures) and `mut` values.

Unlike RefCell they do NOT provide interior mutability! You must have an &mut Ref<T> in
order to get a Mut<T>. However, it is possible to create a reference cycle with these types since `try_borrow`
does not consume the Ref. The kal language itself does not allow this however.

Unlike the guard types from RefCell, Mut shares ownership with the original Ref, allowing the
original Ref to be dropped. In addition, the original Ref can still exist, becoming active again
when the Mut is dropped.

It acts like Rc, and allows aliasing via clone().
If you have a &mut Ref, you can ask to get a Mut to the value. This will not succeed if other
Refs exist.

There cannot be more than one Mut given out by this type.
We only give a Mut when `ref_count == 1`. Therefore, we will not return a Mut
if there are any other Refs to this value before calling `borrow_mut()`.

After creating a Mut with `try_borrow()`, trying to read the inner value of the Ref will fail, as will `clone()`.
*/

use std::{
    fmt::{Debug, Error, Formatter},
    mem, ptr,
};

/// An immutable reference-counted pointer to an inner value.
/// When exactly one Ref exists, mutable actions will succeed.
/// Reading the value is not allowed once a Mut is created.
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

#[allow(unused)]
impl<T> Ref<T> {
    pub fn new(value: T) -> Self {
        Ref {
            ptr: Box::into_raw(Box::new(KcInner {
                borrowed: false,
                ref_count: 1,
                value,
            })),
        }
    }

    /// Clone this Ref
    /// Will not succeed if there is a Mut.
    pub fn try_clone(&self) -> Option<Self> {
        // allowed if there are other Ref but not if there is a Mut
        if self.is_borrowed() {
            None
        } else {
            self.inc_ref_count();
            Some(Self { ptr: self.ptr })
        }
    }

    /// Get a reference to the inner value.
    /// Will not succeed if there is a Mut.
    pub fn try_get(&self) -> Option<&T> {
        // allowed if there are other Ref but not if there is a Mut
        if self.is_borrowed() {
            None
        } else {
            Some(unsafe { &(*self.ptr).value })
        }
    }

    /// Get a mutable reference to the inner value.
    /// Will not succeed if there are other Ref or Mut.
    pub fn try_get_mut(&mut self) -> Option<&mut T> {
        // Only allowed if there are no other Ref
        if self.ref_count() == 1 {
            Some(unsafe { &mut (*self.ptr).value })
        } else {
            None
        }
    }

    /// Turn this Ref into a Mut.
    /// Will not succeed if there are other Ref or Mut.
    pub fn try_into_mut(self) -> Option<Mut<T>> {
        // Only allowed if there are no other Ref
        if self.ref_count() == 1 {
            self.set_borrowed_true();

            // Don't change ref count because we replace this ref with another.
            let r = Some(Mut { ptr: self.ptr });

            std::mem::forget(self);

            r
        } else {
            None
        }
    }

    /// Create a Mut. This Ref will not be usable until the Mut is dropped.
    /// Will not succeed if there are other Ref or Mut.
    pub fn try_borrow(&mut self) -> Option<Mut<T>> {
        // Only allowed if there are no other Ref
        if self.ref_count() == 1 {
            self.set_borrowed_true();
            self.inc_ref_count();
            Some(Mut { ptr: self.ptr })
        } else {
            None
        }
    }

    /// Turn this Ref into the inner value.
    /// Will not succeed if there are other Ref or Mut.
    pub fn try_into_inner(self) -> Result<T, Self> {
        // Only allowed if there are no other Ref
        if self.ref_count() == 1 {
            let inner = unsafe { ptr::read(self.ptr) };
            let value = inner.value;
            mem::forget(self);
            Ok(value)
        } else {
            Err(self)
        }
    }
}

#[allow(unused)]
impl<T> Mut<T> {
    pub fn get(&self) -> &T {
        unsafe { &(*self.ptr).value }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut (*self.ptr).value }
    }

    /// Turn this Mut into a Ref.
    /// If a Ref exists it will become usable again.
    pub fn into_ref(mut self) -> Ref<T> {
        // Unborrow.
        self.set_borrowed_false();

        // Create the new Ref. We destroy this ref when we create the new one, so
        // we don't change the ref count.
        let r = Ref { ptr: self.ptr };

        // don't run drop
        std::mem::forget(self);

        r
    }

    /// Turn this Mut into the inner value.
    /// Will not succeed if a Ref exists.
    pub fn try_into_inner(self) -> Result<T, Self> {
        // Only allowed if there are no other Refs
        if self.ref_count() == 1 {
            let inner = unsafe { ptr::read(self.ptr) };
            let value = inner.value;
            mem::forget(self);
            Ok(value)
        } else {
            Err(self)
        }
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
    fn set_borrowed_true(&self) {
        unsafe {
            (*self.ptr).borrowed = true;
        }
    }
    fn is_borrowed(&self) -> bool {
        unsafe { (*self.ptr).borrowed }
    }
}
impl<T> Mut<T> {
    fn ref_count(&self) -> usize {
        unsafe { (*self.ptr).ref_count }
    }
    fn dec_ref_count(&self) {
        unsafe {
            (*self.ptr).ref_count -= 1;
        }
    }
    fn set_borrowed_false(&self) {
        unsafe {
            (*self.ptr).borrowed = false;
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
        self.set_borrowed_false();
        self.dec_ref_count();
        if self.ref_count() == 0 {
            unsafe { ptr::drop_in_place(self.ptr) }
        }
    }
}

#[doc(hidden)]
struct KcInner<T> {
    borrowed: bool,
    ref_count: usize,
    value: T,
}
