use linux;
use linux::RawFile;
use error::*;
use memory_pressure::MemoryPressureHandle;

#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum IoError {
    Generic = -1,
    Invalid = -2
}

impl IoError {
    pub fn status(&self) -> i32 {
        *self as i32
    }
}

pub type IoResult<T> = Result<T, IoError>;

pub trait Resource {
    /// Memory pressure (should be equivalent to the estimated size in bytes)
    fn init_mem_pressure(&mut self, _pressure: MemoryPressureHandle) {}

    fn read(&mut self, out: &mut [u8]) -> KernelResult<IoResult<usize>>;
    fn write(&mut self, data: &[u8]) -> KernelResult<IoResult<usize>>;
}

pub struct LinuxFile {
    kctx: *mut u8,
    handle: *mut RawFile,
    need_close: bool,
    offset: i64,
    pressure: Option<MemoryPressureHandle>
}

impl Drop for LinuxFile {
    fn drop(&mut self) {
        if self.need_close {
            unsafe {
                linux::lapi_env_close_file(self.handle);
            }
        }
    }
}

impl LinuxFile {
    pub unsafe fn from_raw_checked(kctx: *mut u8, f: *mut RawFile, need_close: bool) -> IoResult<LinuxFile> {
        if f.is_null() {
            Err(IoError::Invalid)
        } else {
            Ok(LinuxFile {
                kctx: kctx,
                handle: f,
                need_close: need_close,
                offset: 0,
                pressure: None
            })
        }
    }
}

impl Resource for LinuxFile {
    fn init_mem_pressure(&mut self, p: MemoryPressureHandle) {
        p.inc(16);
        self.pressure = Some(p);
    }

    fn read(&mut self, out: &mut [u8]) -> KernelResult<IoResult<usize>> {
        let len = out.len();

        if len == 0 {
            return Ok(Ok(0));
        }

        let ret = unsafe {
            linux::lapi_env_read_file(
                self.kctx,
                self.handle,
                &mut out[0],
                len,
                self.offset
            )
        };
        Ok(if ret < 0 {
            Err(IoError::Generic)
        } else {
            self.offset += ret as i64;
            Ok(ret as usize)
        })
    }

    fn write(&mut self, data: &[u8]) -> KernelResult<IoResult<usize>> {
        let len = data.len();

        if len == 0 {
            return Ok(Ok(0));
        }

        let ret = unsafe {
            linux::lapi_env_write_file(
                self.kctx,
                self.handle,
                &data[0],
                len,
                self.offset
            )
        };
        Ok(if ret < 0 {
            Err(IoError::Generic)
        } else {
            self.offset += ret as i64;
            Ok(ret as usize)
        })
    }
}
