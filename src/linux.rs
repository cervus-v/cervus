extern "C" {
    fn lapi_printk(s: *const u8, len: usize);
    pub fn lapi_kmalloc(len: usize) -> *mut u8;
    pub fn lapi_kfree(ptr: *mut u8);
    fn lapi_bug() -> !;

    pub fn lapi_env_reschedule(kctx: *mut u8) -> i32;

    pub fn lapi_env_log(kctx: *mut u8, level: i32, text_base: *const u8, text_len: usize);
    pub fn lapi_env_yield(kctx: *mut u8) -> i32;
    pub fn lapi_env_msleep(kctx: *mut u8, ms: u32) -> i32;
}

macro_rules! println {
    ($fmt:expr) => (::linux::printk(
        &format!($fmt)
    ));
    ($fmt:expr, $($arg:tt)*) => (::linux::printk(
        &format!($fmt, $($arg)*)
    ));
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
