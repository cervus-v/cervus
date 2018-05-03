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

#[repr(C)]
pub struct UserString {
    len: usize,
    data: *const u8
}

#[repr(i32)]
pub enum Command {
    LoadCode = 0x1001,
    RunCode = 0x1002,
    MapCwaApi = 0x1003
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

pub struct ExecEnv<'a> {
    pub args: &'a [&'a str]
}

impl<'a> ExecEnv<'a> {
    pub fn empty() -> ExecEnv<'a> {
        ExecEnv {
            args: &[]
        }
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

    fn submit_code<'a>(
        &mut self,
        code: &[u8],
        backend: Backend,
        cmd: Command,
        exec_env: ExecEnv<'a>
    ) -> ServiceResult<i32> {
        if code.len() == 0 {
            return Err(ServiceError::InvalidInput);
        }

        #[repr(C)]
        struct LoadCodeOptions {
            executor: i32,
            n_args: i32,
            args: *const UserString,
            len: usize,
            addr: *const u8
        }

        let args: Vec<UserString> = exec_env.args.iter()
            .map(|v| {
                let v = v.as_bytes();
                UserString {
                    len: v.len(),
                    data: if v.len() == 0 {
                        ::std::ptr::null()
                    } else {
                        &v[0]
                    }
                }
            })
            .collect();

        let opts = LoadCodeOptions {
            executor: backend as i32,
            n_args: args.len() as i32,
            args: if args.len() > 0 { &args[0] } else { ::std::ptr::null() },
            len: code.len(),
            addr: &code[0]
        };

        match cmd {
            Command::LoadCode | Command::RunCode => {},
            _ => {
                return Err(ServiceError::InvalidInput);
            }
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
        match self.submit_code(code, backend, Command::LoadCode, ExecEnv::empty()) {
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

    pub fn run_code(
        &mut self,
        code: &[u8],
        backend: Backend,
        args: &[&str]
    ) -> ServiceResult<i32> {
        self.submit_code(code, backend, Command::RunCode, ExecEnv {
            args: args
        })
    }

    pub fn map_cwa_api(&self, name: &str) -> Option<u32> {
        #[repr(C)]
        struct Request {
            name: *const u8,
            len: usize
        }

        let name = name.as_bytes();
        if name.len() == 0 {
            return None;
        }

        let req = Request {
            name: &name[0],
            len: name.len()
        };
        let fd = self.dev.as_raw_fd();
        let ret = unsafe {
            ::libc::ioctl(
                fd,
                Command::MapCwaApi as i32 as ::libc::c_ulong,
                &req as *const Request as ::libc::c_ulong
            )
        };

        if ret < 0 {
            None
        } else {
            Some(ret as u32)
        }
    }
}
