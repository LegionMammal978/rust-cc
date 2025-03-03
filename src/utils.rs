use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ptr::NonNull;

use crate::{state, CcOnHeap, Trace};

#[inline]
pub(crate) unsafe fn cc_alloc<T: Trace + 'static>(layout: Layout) -> NonNull<CcOnHeap<T>> {
    state(|state| state.record_allocation(layout));
    match NonNull::new(alloc(layout) as *mut CcOnHeap<T>) {
        Some(ptr) => ptr,
        None => handle_alloc_error(layout),
    }
}

#[inline]
pub(crate) unsafe fn cc_dealloc<T: ?Sized + Trace + 'static>(
    ptr: NonNull<CcOnHeap<T>>,
    layout: Layout,
) {
    state(|state| state.record_deallocation(layout));
    dealloc(ptr.cast().as_ptr(), layout)
}

#[inline(always)]
#[cold]
pub(crate) fn cold() {}
