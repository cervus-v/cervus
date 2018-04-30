use linux;
use backend::common::*;

pub struct UsermodeContext {
    kctx: *mut u8
}

impl UsermodeContext {
    pub fn new(kctx: *mut u8) -> UsermodeContext {
        UsermodeContext {
            kctx: kctx
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
        if start >= self.len() || end > self.len() || end < start {
            return Err(BackendError::Bounds);
        }

        match ::core::str::from_utf8(&self[start .. end]) {
            Ok(v) => Ok(v),
            Err(_) => Err(BackendError::Generic)
        }
    }
}
