extern "C" {
    fn lapi_printk(s: *const u8, len: usize);
    pub fn lapi_kmalloc(len: usize) -> *mut u8;
    pub fn lapi_kfree(ptr: *mut u8);
    fn lapi_bug() -> !;

    pub fn lapi_env_reschedule(kctx: *mut u8) -> i32;

    pub fn lapi_env_get_uid(kctx: *mut u8) -> i32;

    pub fn lapi_semaphore_new() -> *mut RawSemaphore;
    pub fn lapi_semaphore_destroy(sem: *mut RawSemaphore);
    pub fn lapi_semaphore_up(sem: *mut RawSemaphore);
    pub fn lapi_semaphore_down(sem: *mut RawSemaphore) -> i32;

    pub fn lapi_oom_score_adj_current(score: i16);
    pub fn lapi_get_total_ram_bytes() -> usize;

    pub fn lapi_env_get_n_args(kctx: *mut u8) -> u32;
    pub fn lapi_env_read_arg(kctx: *mut u8, id: u32, out: *mut u8, max_len: usize) -> isize;

    pub fn lapi_env_open_file(
        kctx: *mut u8,
        name_base: *const u8,
        name_len: usize,
        flags_base: *const u8,
        flags_len: usize
    ) -> *mut RawFile;

    pub fn lapi_env_close_file(file: *mut RawFile);

    pub fn lapi_env_write_file(
        kctx: *mut u8,
        file: *mut RawFile,
        data: *const u8,
        len: usize,
        offset: i64
    ) -> isize;

    pub fn lapi_env_read_file(
        kctx: *mut u8,
        file: *mut RawFile,
        data: *mut u8,
        len: usize,
        offset: i64
    ) -> isize;

    pub fn lapi_env_get_stdin(
        kctx: *mut u8
    ) -> *mut RawFile;

    pub fn lapi_env_get_stdout(
        kctx: *mut u8
    ) -> *mut RawFile;

    pub fn lapi_env_get_stderr(
        kctx: *mut u8
    ) -> *mut RawFile;

    pub fn lapi_env_log(kctx: *mut u8, level: i32, text_base: *const u8, text_len: usize);
    pub fn lapi_env_yield(kctx: *mut u8) -> i32;
    pub fn lapi_env_msleep(kctx: *mut u8, ms: u32) -> i32;
}

#[repr(C)]
pub struct RawFile {
    _opaque: usize
}

#[repr(C)]
pub struct RawSemaphore {
    _opaque: usize
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
