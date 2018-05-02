static TARGET_VERSION: &'static str = "0.1.0";

#[wasm_import_module = "hexagon_e"]
extern "C" {
    fn syscall_0(level: i32, text_base: *const u8, text_len: usize); // log
    fn syscall_100000(version_base: *const u8, version_len: usize); // chk_version
    fn syscall_100001(); // yield
    fn syscall_100002(ms: u32); // msleep
    fn syscall_100003(fd: i32, addr: *mut u8, len: usize) -> i32; // read
    fn syscall_100004(fd: i32, addr: *const u8, len: usize) -> i32; // write
    fn syscall_100005(fd: i32); // close
    fn syscall_100006() -> i32; // get_n_args
    fn syscall_100007(id: u32, addr: *mut u8, len: usize) -> i32; // read_arg

    fn syscall_110000() -> i32; // get_stdin
    fn syscall_110001() -> i32; // get_stdout
    fn syscall_110002() -> i32; // get_stderr
}

pub fn get_n_args() -> usize {
    unsafe { syscall_100006() as usize }
}

pub fn read_arg(id: u32, out: &mut [u8]) -> isize {
    let len = out.len();

    if len == 0 {
        -1
    } else {
        unsafe { syscall_100007(id, &mut out[0], len) as isize }
    }
}

pub fn get_stdin() -> i32 {
    unsafe { syscall_110000() }
}

pub fn get_stdout() -> i32 {
    unsafe { syscall_110001() }
}

pub fn get_stderr() -> i32 {
    unsafe { syscall_110002() }
}

pub fn read(fd: i32, data: &mut [u8]) -> i32 {
    let len = data.len();

    if len == 0 {
        -1
    } else {
        unsafe {
            syscall_100003(fd, &mut data[0], len)
        }
    }
}

pub fn write(fd: i32, data: &[u8]) -> i32 {
    let len = data.len();

    if len == 0 {
        -1
    } else {
        unsafe {
            syscall_100004(fd, &data[0], len)
        }
    }
}

pub fn close(fd: i32) {
    unsafe {
        syscall_100005(fd)
    }
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
            ::std::ptr::null()
        }, text.len());
    }
}
