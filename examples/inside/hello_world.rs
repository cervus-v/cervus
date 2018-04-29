#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(wasm_import_module)]
#![no_std]

#[wasm_import_module = "hexagon_e"]
extern "C" {
    fn syscall_0(level: i32, text_base: *const u8, text_len: usize);
}

#[no_mangle]
pub extern "C" fn run() {
    static TEXT: &'static str = "Hello world!";

    let text = TEXT.as_bytes();
    unsafe {
        syscall_0(1, &text[0], text.len());
    }
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(_args: core::fmt::Arguments, _file: &'static str, _line: u32) -> ! {
    unsafe { ::core::intrinsics::abort(); }
}
