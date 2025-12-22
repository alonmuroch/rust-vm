use core::cell::UnsafeCell;

/// Minimal wrapper to store non-`Sync` types in statics.
///
/// Safety: Callers must guarantee exclusive access when mutating.
pub struct Global<T> {
    inner: UnsafeCell<T>,
}

impl<T> Global<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    /// # Safety
    /// Callers must ensure exclusive access or otherwise serialize mutations.
    pub unsafe fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }
}

unsafe impl<T> Sync for Global<T> {}
