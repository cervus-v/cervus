#![feature(lang_items)]
#![no_std]

extern crate hexagon_e;

pub mod linux;
pub mod env;

use hexagon_e::error::*;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(_args: core::fmt::Arguments, _file: &'static str, _line: u32) -> ! {
    linux::kernel_panic(_file);
}

pub extern "C" fn run(
    code_base: *const u8,
    code_len: usize,
    mem_default_len: usize,
    mem_max_len: usize,
    stack_len: usize,
    call_stack_len: usize,
    kctx: *mut u8
) -> i32 {
    fn run_inner(code: &[u8], config: env::EnvConfig) -> ExecuteResult<()> {
        let m = hexagon_e::module::Module::from_raw(code)?;
        let mut rh = match env::ResourceHolder::new(config) {
            Some(v) => v,
            None => return Err(ExecuteError::Generic)
        };
        let env = env::ExecutionEnv::new(&mut rh);

        let mut vm = hexagon_e::vm::VirtualMachine::new(&m, env);
        vm.run_memory_initializers()?;
        vm.run()?;

        Ok(())
    }

    let code = unsafe { ::core::slice::from_raw_parts(code_base, code_len) };
    let config = env::EnvConfig {
        kctx: kctx,
        memory_default_len: mem_default_len,
        memory_max_len: mem_max_len,
        stack_len: stack_len,
        call_stack_len: call_stack_len
    };

    let result = run_inner(code, config);

    match result {
        Ok(_) => 0,
        Err(_) => -1
    }
}
