use alloc::boxed::Box;

use linux;
use backend::common::*;
use slab::Slab;
use resource::Resource;
use resource::LinuxFile;
use error::*;

pub struct UsermodeContext {
    pub kctx: *mut u8,
    pub resources: Slab<Box<Resource>>,
    mem_pressure: usize
}

impl UsermodeContext {
    pub fn new(kctx: *mut u8) -> UsermodeContext {
        UsermodeContext {
            kctx: kctx,
            resources: Slab::new(),
            mem_pressure: 0
        }
    }

    pub fn map_cwa_api_to_native_invoke(name: &str) -> Option<u32> {
        ::global::get_global().native_invoke_registry.map_name_to_id(name)
    }

    pub fn add_resource(&mut self, res: Box<Resource>) -> usize {
        self.mem_pressure += res.mem_pressure();
        self.resources.insert(res)
    }

    pub fn remove_resource(&mut self, id: usize) -> KernelResult<()> {
        let res = self.resources.remove(id)?;
        self.mem_pressure -= res.mem_pressure();
        Ok(())
    }

    pub unsafe fn add_raw_linux_file(&mut self, raw: *mut linux::RawFile, need_close: bool) -> i32 {
        match LinuxFile::from_raw_checked(
            self.kctx,
            raw,
            need_close
        ) {
            Ok(v) => self.add_resource(Box::new(v)) as i32,
            Err(_) => -1
        }
    }

    pub fn log(&self, level: i32, text: &str) {
        let text = text.as_bytes();

        unsafe { linux::lapi_env_log(
            self.kctx,
            level,
            if text.len() == 0 { ::core::ptr::null() } else { &text[0] },
            text.len()
        ); }
    }
}

impl Context for UsermodeContext {
    fn tick(&self) -> BackendResult<()> {
        let ret = unsafe { linux::lapi_env_reschedule(self.kctx) };
        if ret < 0 {
            Err(BackendError::FatalSignal)
        } else {
            Ok(())
        }
    }

    fn get_native_invoke_policy(&self, id: usize) -> BackendResult<NativeInvokePolicy> {
        Ok(::global::get_global().native_invoke_registry.get(id)?.policy())
    }

    fn do_native_invoke(&mut self, id: usize, args: &[i64], mem: &mut [u8]) -> BackendResult<Option<i64>> {
        ::global::get_global().native_invoke_registry.get(id)?.call(self, args, mem)
    }
}
