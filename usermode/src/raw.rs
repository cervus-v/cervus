static TARGET_VERSION: &'static str = "0.1.0";

#[wasm_import_module = "hexagon_e"]
extern "C" {
    fn syscall_0(level: i32, text_base: *const u8, text_len: usize); // log
    fn syscall_100000(version_base: *const u8, version_len: usize); // chk_version
    fn syscall_100001(); // yield
    fn syscall_100002(ms: u32); // msleep
}

pub fn chk_version() {
    let version = TARGET_VERSION.as_bytes();
    unsafe {
        syscall_100000(&version[0], version.len());
    }
}

pub fn yield_now() {
    unsafe {
        syscall_100001();
    }
}

pub fn msleep(ms: u32) {
    unsafe {
        syscall_100002(ms);
    }
}

pub enum LogLevel {
    Error,
    Warning,
    Info
}

#[inline(always)]
pub fn log(level: LogLevel, text: &str) {
    let level: i32 = match level {
        LogLevel::Error => 1,
        LogLevel::Warning => 3,
        LogLevel::Info => 6
    };

    let text = text.as_bytes();

    unsafe {
        syscall_0(level, if text.len() > 0 {
            &text[0]
        } else {
            ::core::ptr::null()
        }, text.len());
    }
}
