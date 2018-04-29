use core::ops::{Deref, DerefMut};

extern "C" {
    fn lapi_printk(s: *const u8, len: usize);
    fn lapi_kmalloc(len: usize) -> *mut u8;
    fn lapi_kfree(ptr: *mut u8);
    fn lapi_bug() -> !;

    pub fn lapi_env_log(kctx: *mut u8, level: i32, text_base: *const u8, text_len: usize);
}

pub fn printk(s: &str) {
    let bytes = s.as_bytes();
    unsafe {
        if bytes.len() == 0 {
            lapi_printk(::core::ptr::null(), 0);
        } else {
            lapi_printk(&bytes[0], bytes.len());
        }
    }
}

pub fn kernel_panic(why: &str) -> ! {
    printk(why);
    unsafe { lapi_bug() }
}

pub struct BoxedSlice<T> {
    mem: *mut [T]
}

impl<T> BoxedSlice<T> {
    pub fn new<F: Fn() -> T>(value_feed: F, len: usize) -> Option<BoxedSlice<T>> {
        if len == 0 {
            return Some(BoxedSlice {
                mem: unsafe { ::core::slice::from_raw_parts_mut(::core::ptr::null_mut(), 0) }
            });
        }
        use core::mem::size_of;

        let byte_len = len * size_of::<T>();
        let mem = unsafe { lapi_kmalloc(byte_len) } as *mut T;
        if mem.is_null() {
            return None;
        }

        let slice = unsafe { ::core::slice::from_raw_parts_mut(mem, len) };

        // FIXME: Unwind safety
        for i in 0..len {
            unsafe { ::core::ptr::write(&mut slice[i], value_feed()); }
        }

        Some(BoxedSlice {
            mem: slice
        })
    }
}

impl<T> Deref for BoxedSlice<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { &*self.mem }
    }
}

impl<T> DerefMut for BoxedSlice<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { &mut *self.mem }
    }
}

impl<T> Drop for BoxedSlice<T> {
    fn drop(&mut self) {
        unsafe {
            let mem = &mut *self.mem;
            if mem.len() == 0 {
                return;
            }
            for elem in mem.iter_mut() {
                ::core::ptr::drop_in_place(elem);
            }
            lapi_kfree((&mut mem[0]) as *mut T as *mut u8);
        }
    }
}
