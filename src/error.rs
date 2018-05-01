pub enum KernelError {
    NoMem,
    FatalSignal
}

pub type KernelResult<T> = Result<T, KernelError>;
