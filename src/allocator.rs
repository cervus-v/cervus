use core::ptr::NonNull;
use core::alloc::Opaque;
use alloc::allocator::GlobalAlloc;
use alloc::allocator::{Alloc, Layout, AllocErr};
use linux;

pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        let size = layout.size() + layout.padding_needed_for(layout.align());
        if size == 0 {
            ::core::mem::zeroed()
        } else {
            linux::lapi_kmalloc(size) as *mut Opaque
        }
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, _layout: Layout) {
        linux::lapi_kfree(ptr as *mut u8);
    }
}

unsafe impl Alloc for KernelAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<Opaque>, AllocErr> {
        NonNull::new(GlobalAlloc::alloc(self, layout)).ok_or(AllocErr)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<Opaque>, layout: Layout) {
        GlobalAlloc::dealloc(self, ptr.as_ptr(), layout)
    }
}
