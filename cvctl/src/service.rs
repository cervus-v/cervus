use std::fs::File;
use std::io;
use std::error::Error;
use std::os::unix::io::AsRawFd;

macro_rules! impl_debug_display {
    ($target:ident) => {
        impl ::std::fmt::Display for $target {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                <Self as ::std::fmt::Debug>::fmt(self, f)
            }
        }
    }
}

#[repr(i32)]
pub enum Command {
    LoadCode = 0x1001,
    RunCode = 0x1002
}

#[repr(i32)]
pub enum Backend {
    HexagonE = 0x01
}

#[derive(Debug)]
pub enum ServiceError {
    Io(io::Error),
    InvalidInput,
    Rejected
}

pub type ServiceResult<T> = Result<T, ServiceError>;

impl_debug_display!(ServiceError);

impl Error for ServiceError {
    fn description(&self) -> &str {
        "ServiceError"
    }
}

impl From<io::Error> for ServiceError {
    fn from(other: io::Error) -> ServiceError {
        ServiceError::Io(other)
    }
}

pub struct ServiceContext {
    dev: File
}

impl ServiceContext {
    pub fn connect() -> ServiceResult<ServiceContext> {
        Ok(ServiceContext {
            dev: File::open("/dev/cvctl")?
        })
    }

    fn submit_code(&mut self, code: &[u8], backend: Backend, cmd: Command) -> ServiceResult<i32> {
        if code.len() == 0 {
            return Err(ServiceError::InvalidInput);
        }

        #[repr(C)]
        struct LoadCodeOptions {
            executor: i32,
            len: usize,
            addr: *const u8
        }

        let opts = LoadCodeOptions {
            executor: backend as i32,
            len: code.len(),
            addr: &code[0]
        };

        match cmd {
            Command::LoadCode | Command::RunCode => {},
            /*_ => {
                return Err(ServiceError::InvalidInput);
            }*/
        }

        let fd = self.dev.as_raw_fd();
        let ret = unsafe {
            ::libc::ioctl(
                fd,
                cmd as i32 as ::libc::c_ulong,
                &opts as *const LoadCodeOptions as ::libc::c_ulong
            )
        };

        Ok(ret)
    }

    pub fn load_code(&mut self, code: &[u8], backend: Backend) -> ServiceResult<()> {
        match self.submit_code(code, backend, Command::LoadCode) {
            Ok(v) => {
                if v == 0 {
                    Ok(())
                } else {
                    Err(ServiceError::Rejected)
                }
            },
            Err(e) => Err(e)
        }
    }

    pub fn run_code(&mut self, code: &[u8], backend: Backend) -> ServiceResult<i32> {
        self.submit_code(code, backend, Command::RunCode)
    }
}
