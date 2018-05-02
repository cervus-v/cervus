use alloc::boxed::Box;

use linux;
use backend::common::*;
use slab::Slab;
use resource::Resource;
use resource::LinuxFile;
use error::*;

pub struct UsermodeContext {
    kctx: *mut u8,
    resources: Slab<Box<Resource>>,
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

    fn add_resource(&mut self, res: Box<Resource>) -> usize {
        self.mem_pressure += res.mem_pressure();
        self.resources.insert(res)
    }

    fn remove_resource(&mut self, id: usize) -> KernelResult<()> {
        let res = self.resources.remove(id)?;
        self.mem_pressure -= res.mem_pressure();
        Ok(())
    }

    unsafe fn add_raw_linux_file(&mut self, raw: *mut linux::RawFile) -> i32 {
        match LinuxFile::from_raw_checked(
            self.kctx,
            raw
        ) {
            Ok(v) => self.add_resource(Box::new(v)) as i32,
            Err(_) => -1
        }
    }

    fn log(&self, level: i32, text: &str) {
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
        match id {
            0 => Ok(NativeInvokePolicy { n_args: 3 }),
            100000 => Ok(NativeInvokePolicy { n_args: 2 }),
            100001 => Ok(NativeInvokePolicy { n_args: 0 }),
            100002 => Ok(NativeInvokePolicy { n_args: 1 }),
            100003 => Ok(NativeInvokePolicy { n_args: 3 }),
            100004 => Ok(NativeInvokePolicy { n_args: 3 }),
            100005 => Ok(NativeInvokePolicy { n_args: 1 }),
            110000 => Ok(NativeInvokePolicy { n_args: 0 }),
            110001 => Ok(NativeInvokePolicy { n_args: 0 }),
            110002 => Ok(NativeInvokePolicy { n_args: 0 }),
            _ => Err(BackendError::InvalidNativeInvoke)
        }
    }

    fn do_native_invoke(&mut self, id: usize, args: &[i64], mem: &mut [u8]) -> BackendResult<Option<i64>> {
        match id {
            0 => {
                check_len(args, 3)?;
                let level = args[0] as i32;
                let text_base = args[1] as u32 as usize;
                let text_len = args[2] as u32 as usize;
                let text = mem.extract_str(text_base, text_len)?;
                self.log(level, text);
                Ok(None)
            },
            100000 /* chk_version */ => {
                static VERSION: &'static str = env!("CARGO_PKG_VERSION");

                check_len(args, 2)?;
                let text_base = args[0] as u32 as usize;
                let text_len = args[1] as u32 as usize;

                let client_version = mem.extract_str(text_base, text_len)?;
                if client_version == VERSION {
                    Ok(None)
                } else {
                    self.log(1, "Version mismatch");
                    Err(BackendError::Generic)
                }
            },
            100001 /* yield */ => {
                check_len(args, 0)?;
                let ret = unsafe { linux::lapi_env_yield(self.kctx) };

                if ret < 0 {
                    Err(BackendError::FatalSignal)
                } else {
                    Ok(None)
                }
            },
            100002 /* msleep */ => {
                check_len(args, 1)?;
                let ms = args[0] as u32;
                let ret = unsafe { linux::lapi_env_msleep(self.kctx, ms) };

                if ret < 0 {
                    Err(BackendError::FatalSignal)
                } else {
                    Ok(None)
                }
            },
            100003 /* read */ => {
                check_len(args, 3)?;

                let id = args[0] as u32 as usize;
                let mem_begin = args[1] as u32 as usize;
                let len = args[2] as u32 as usize;

                let out = mem.checked_slice_mut(mem_begin, mem_begin + len)?;
                self.resources.get_mut(id)?.read(out)
                    .map(|n| Some(n as i64))
                    .or_else(|_| Ok(Some(-1)))
            },
            100004 /* write */ => {
                check_len(args, 3)?;

                let id = args[0] as u32 as usize;
                let mem_begin = args[1] as u32 as usize;
                let len = args[2] as u32 as usize;

                let data = mem.checked_slice(mem_begin, mem_begin + len)?;
                self.resources.get_mut(id)?.write(data)
                    .map(|n| Some(n as i64))
                    .or_else(|_| Ok(Some(-1)))
            },
            100005 /* close */ => {
                check_len(args, 1)?;

                let id = args[0] as u32 as usize;
                self.remove_resource(id)?;

                Ok(None)
            },
            110000 /* get_stdin */ => {
                check_len(args, 0)?;
                let kctx = self.kctx;
                Ok(Some(unsafe {
                    self.add_raw_linux_file(linux::lapi_env_get_stdin(kctx))
                 } as i64))
            },
            110001 /* get_stdout */ => {
                check_len(args, 0)?;
                let kctx = self.kctx;
                Ok(Some(unsafe {
                    self.add_raw_linux_file(linux::lapi_env_get_stdout(kctx))
                 } as i64))
            },
            110002 /* get_stderr */ => {
                check_len(args, 0)?;
                let kctx = self.kctx;
                Ok(Some(unsafe {
                    self.add_raw_linux_file(linux::lapi_env_get_stderr(kctx))
                 } as i64))
            },
            _ => Err(BackendError::InvalidNativeInvoke)
        }
    }
}

fn check_len<T>(a: &[T], expected: usize) -> BackendResult<()> {
    if a.len() == expected {
        Ok(())
    } else {
        Err(BackendError::InvalidInput)
    }
}

trait ExtractStr {
    fn extract_str(&self, start: usize, len: usize) -> BackendResult<&str>;
}

impl ExtractStr for [u8] {
    fn extract_str(&self, start: usize, len: usize) -> BackendResult<&str> {
        let end = start + len;
        let data = self.checked_slice(start, end)?;

        match ::core::str::from_utf8(data) {
            Ok(v) => Ok(v),
            Err(_) => Err(BackendError::Generic)
        }
    }
}

trait CheckedSlice {
    type Target;
    fn checked_slice(&self, start: usize, end: usize) -> BackendResult<&[Self::Target]>;
    fn checked_slice_mut(&mut self, start: usize, end: usize) -> BackendResult<&mut [Self::Target]>;
}

impl<T> CheckedSlice for [T] {
    type Target = T;
    fn checked_slice(&self, start: usize, end: usize) -> BackendResult<&[Self::Target]> {
        if start > end || start > self.len() || end > self.len() {
            Err(BackendError::Bounds)
        } else {
            Ok(&self[start..end])
        }
    }
    fn checked_slice_mut(&mut self, start: usize, end: usize) -> BackendResult<&mut [Self::Target]> {
        if start > end || start > self.len() || end > self.len() {
            Err(BackendError::Bounds)
        } else {
            Ok(&mut self[start..end])
        }
    }
}
