use linux;
use linux::RawFile;

pub enum IoError {
    Generic,
    Invalid
}

pub type IoResult<T> = Result<T, IoError>;

pub trait Resource {
    fn mem_pressure(&self) -> usize;
    fn read(&mut self, out: &mut [u8]) -> IoResult<usize>;
    fn write(&mut self, data: &[u8]) -> IoResult<usize>;
}

pub struct LinuxFile {
    kctx: *mut u8,
    handle: *mut RawFile,
    offset: i64
}

impl LinuxFile {
    pub unsafe fn from_raw_checked(kctx: *mut u8, f: *mut RawFile) -> IoResult<LinuxFile> {
        if f.is_null() {
            Err(IoError::Invalid)
        } else {
            Ok(LinuxFile {
                kctx: kctx,
                handle: f,
                offset: 0
            })
        }
    }
}

impl Resource for LinuxFile {
    fn mem_pressure(&self) -> usize {
        5
    }

    fn read(&mut self, out: &mut [u8]) -> IoResult<usize> {
        let len = out.len();

        if len == 0 {
            return Ok(0);
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
        if ret < 0 {
            Err(IoError::Generic)
        } else {
            self.offset += ret as i64;
            Ok(ret as usize)
        }
    }

    fn write(&mut self, data: &[u8]) -> IoResult<usize> {
        let len = data.len();

        if len == 0 {
            return Ok(0);
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
        if ret < 0 {
            Err(IoError::Generic)
        } else {
            self.offset += ret as i64;
            Ok(ret as usize)
        }
    }
}
