use alloc::boxed::Box;
use core::cell::Cell;

use linux;
use backend::common::*;
use slab::Slab;
use resource::Resource;
use resource::LinuxFile;
use memory_pressure::MemoryPressure;
use error::*;

pub struct UsermodeContext {
    pub kctx: *mut u8,
    pub resources: Slab<Box<Resource>>,
    mp: MemoryPressure,
    prev_oom_score_adj: Cell<i16>
}

fn calc_oom_score_adj(mem_pressure: usize) -> i16 {
    let total_memory = ::global::get_global().total_memory;

    if mem_pressure >= total_memory {
        1000
    } else {
        (mem_pressure * 1000 / total_memory) as i16
    }
}

impl UsermodeContext {
    pub fn new(kctx: *mut u8) -> UsermodeContext {
        UsermodeContext {
            kctx: kctx,
            resources: Slab::new(),
            mp: MemoryPressure::new(),
            prev_oom_score_adj: Cell::new(0)
        }
    }

    pub fn map_cwa_api_to_native_invoke(name: &str) -> Option<u32> {
        ::global::get_global().native_invoke_registry.map_name_to_id(name)
    }

    fn update_oom_score(&self) {
        let new_val = calc_oom_score_adj(self.mp.read());
        let old_val = self.prev_oom_score_adj.get();

        if new_val != old_val {
            unsafe { linux::lapi_oom_score_adj_current(new_val) };
            self.prev_oom_score_adj.set(new_val);
        }
    }

    pub fn add_resource(&mut self, mut res: Box<Resource>) -> usize {
        res.init_mem_pressure(self.mp.handle());
        self.update_oom_score();

        self.resources.insert(res)
    }

    pub fn remove_resource(&mut self, id: usize) -> KernelResult<()> {
        self.resources.remove(id)?;
        self.update_oom_score();

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
        let result = ::global::get_global().native_invoke_registry.get(id)?.call(self, args, mem);
        self.update_oom_score();
        result
    }
}
