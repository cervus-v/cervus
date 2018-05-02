pub enum KernelError {
    Generic,
    NoMem,
    FatalSignal,
    InvalidResource
}

pub type KernelResult<T> = Result<T, KernelError>;
