macro_rules! impl_ni_common {
    ($name:ident, n_args = $n_args:expr, ($ctx_ident:ident, $args_ident:ident, $mem_ident:ident) => $call_blk:block) => {
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl ::api::NativeInvoke for $name {
            fn name(&self) -> &'static str { stringify!($name) }
            fn policy(&self) -> ::backend::common::NativeInvokePolicy { ::backend::common::NativeInvokePolicy {
                n_args: $n_args
            } }
            fn call(
                &self,
                $ctx_ident: &mut ::env::UsermodeContext,
                $args_ident: &[i64],
                $mem_ident: &mut [u8]
            ) -> ::backend::common::BackendResult<Option<i64>> {
                ::api::check_len($args_ident, $n_args)?;
                $call_blk
            }
        }
    }
}

mod runtime;
mod log;
mod env;
mod startup;
mod resource;
mod io;
mod ipc;

use alloc::BTreeMap;
use alloc::boxed::Box;
use alloc::Vec;
use backend::common::*;
use env::UsermodeContext;

pub trait NativeInvoke: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn policy(&self) -> NativeInvokePolicy;
    fn call(&self, context: &mut UsermodeContext, args: &[i64], mem: &mut [u8]) -> BackendResult<Option<i64>>;
}

#[derive(Default)]
pub struct Registry {
    targets: Vec<Box<NativeInvoke>>,
    name_mappings: BTreeMap<&'static str, u32>,
}

impl Registry {
    pub fn new() -> Registry {
        let mut reg = Registry::default();

        reg.register(runtime::runtime_spec_major);
        reg.register(runtime::runtime_spec_minor);
        reg.register(runtime::runtime_name);
        reg.register(runtime::runtime_msleep);
        reg.register(log::log_write);
        reg.register(env::env_get);
        reg.register(startup::startup_arg_len);
        reg.register(startup::startup_arg_at);
        reg.register(resource::resource_read);
        reg.register(resource::resource_write);
        reg.register(resource::resource_open);
        reg.register(resource::resource_close);
        reg.register(io::io_get_stdin);
        reg.register(io::io_get_stdout);
        reg.register(io::io_get_stderr);

        reg
    }

    pub fn map_name_to_id(&self, name: &str) -> Option<u32> {
        self.name_mappings.get(name).map(|v| *v)
    }

    pub fn get(&self, id: usize) -> BackendResult<&NativeInvoke> {
        if id >= self.targets.len() {
            Err(BackendError::InvalidNativeInvoke)
        } else {
            Ok(&*self.targets[id])
        }
    }

    fn register<T: NativeInvoke>(&mut self, ni: T) {
        let name = ni.name();
        let id = self.targets.len() as u32;

        self.targets.push(Box::new(ni));
        self.name_mappings.insert(name, id);
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
