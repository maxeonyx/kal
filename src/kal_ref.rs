use std::ops::Deref;

pub struct KalRef<T> {
    ptr: *mut KalRefInner<T>,
}

impl<T> KalRef<T> {
    pub fn alias(&self) -> Self {
        *self.ref_count() += 1;
        Self { ptr: self.ptr }
    }

    pub fn new(value: T) -> Self {
        KalRef {
            ptr: Box::into_raw(Box::new(KalRefInner {
                ref_count: 1,
                value,
            })),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if *self.ref_count() > 1 {
            None
        } else {
            Some(unsafe { &mut (*self.ptr).value })
        }
    }
}

impl<T> Deref for KalRef<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &(*self.ptr).value }
    }
}

impl<T> KalRef<T> {
    fn ref_count(&self) -> &mut u64 {
        unsafe { &mut (*self.ptr).ref_count }
    }
}

impl<T> Drop for KalRef<T> {
    fn drop(&mut self) {
        *self.ref_count() -= 1;
        if *self.ref_count() == 0 {
            std::ptr::drop_in_place(self.ptr)
        }
    }
}

struct KalRefInner<T> {
    ref_count: u64,
    value: T,
}
