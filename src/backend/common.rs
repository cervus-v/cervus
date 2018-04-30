#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum BackendError {
    Generic = 1,
    Bounds,
    InvalidNativeInvoke,
    NotFound,
    InvalidInput,
    FatalSignal
}

impl BackendError {
    pub fn status(&self) -> i32 {
        -(*self as u8 as i32)
    }
}

pub type BackendResult<T> = Result<T, BackendError>;

pub trait Backend: Sized {
    type Config;

    fn new(config: Self::Config) -> BackendResult<Self>;
    fn run<C: Context>(
        &mut self,
        code: &[u8],
        context: &mut C
    ) -> BackendResult<()>;
}

pub struct NativeInvokePolicy {
    pub n_args: usize
}

pub trait Context {
    fn get_native_invoke_policy(&self, id: usize) -> BackendResult<NativeInvokePolicy>;
    fn do_native_invoke(&mut self, id: usize, args: &[i64], mem: &mut [u8]) -> BackendResult<Option<i64>>;
    fn tick(&self) -> BackendResult<()>;
}
