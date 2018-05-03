pub enum KernelError {
    Generic,
    NoMem,
    FatalSignal,
    InvalidResource
}

pub type KernelResult<T> = Result<T, KernelError>;

#[repr(i32)]
#[derive(Copy, Clone, Debug)]
pub enum CwaError {
    Unknown = -1,
    InvalidArgument = -2,
    PermissionDenied = -3,
    NotFound = -4
}

pub type CwaResult<T> = Result<T, CwaError>;

impl CwaError {
    pub fn status(&self) -> i32 {
        *self as i32
    }
}
