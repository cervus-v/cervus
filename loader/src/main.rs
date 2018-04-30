extern crate wasm_core;
extern crate libc;

use std::fs::File;
use std::env;
use std::io::Read;
use std::os::unix::io::AsRawFd;

use wasm_core::trans::config::ModuleConfig;
use wasm_core::hetrans::translate_module;

#[repr(C)]
struct LoadCodeInfo {
    executor: i32,
    len: usize,
    addr: *const u8
}

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    let path = args.next().expect("Path required");
    let entry_fn_name = args.next().expect("Entry function required");
    let mut f = File::open(&path).unwrap();
    let mut code: Vec<u8> = Vec::new();

    let cfg: ModuleConfig = ModuleConfig::default();

    f.read_to_end(&mut code).unwrap();
    let module = wasm_core::trans::translate_module_raw(code.as_slice(), cfg);
    let entry_fn = module.lookup_exported_func(&entry_fn_name).expect("Entry function not found");

    let result = translate_module(&module, entry_fn);
    let loader: File = File::open("/dev/cvctl").unwrap();

    eprintln!("Code length: {}", result.len());
    let fd = loader.as_raw_fd();

    let load_opt = LoadCodeInfo {
        executor: 1,
        len: result.len(),
        addr: &result[0]
    };
    let ret = unsafe {
        libc::ioctl(fd, 1, &load_opt as *const LoadCodeInfo as usize)
    };
    assert!(ret >= 0);
}
