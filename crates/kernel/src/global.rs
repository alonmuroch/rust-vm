extern crate alloc;

use alloc::vec::Vec;
use core::cell::UnsafeCell;
use state::State;

use crate::{BootInfo, Task};
use crate::mmu::PageAllocator;

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

#[allow(dead_code)]
pub static TASKS: Global<Option<Vec<Task>>> = Global::new(None);
pub static STATE: Global<Option<State>> = Global::new(None);
pub static BOOT_INFO: Global<Option<BootInfo>> = Global::new(None);
pub static PAGE_ALLOC_INIT: Global<bool> = Global::new(false);
pub static NEXT_ASID: Global<u16> = Global::new(1);
pub static ROOT_PPN: Global<u32> = Global::new(0);
pub static PAGE_ALLOC: Global<Option<PageAllocator>> = Global::new(None);
