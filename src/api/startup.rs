use super::*;
use linux;
use error::CwaError;

impl_ni_common!(
    startup_arg_len,
    n_args = 0,
    (ctx, _args, _mem) => {
        let n = unsafe {
            linux::lapi_env_get_n_args(ctx.kctx)
        };

        Ok(Some(n as i64))
    }
);

impl_ni_common!(
    startup_arg_at,
    n_args = 3,
    (ctx, args, mem) => {
        let id = args[0] as u32;
        let mem_begin = args[1] as u32 as usize;
        let len = args[2] as u32 as usize;

        let out = mem.checked_slice_mut(mem_begin, mem_begin + len)?;
        if out.len() == 0 {
            Err(BackendError::InvalidInput)
        } else {
            let ret = unsafe {
                linux::lapi_env_read_arg(
                    ctx.kctx,
                    id,
                    &mut out[0],
                    out.len()
                )
            };
            Ok(Some(if ret >= 0 {
                ret as i64
            } else {
                CwaError::InvalidArgument.status() as i64
            }))
        }
    }
);
