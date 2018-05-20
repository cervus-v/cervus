use super::*;
use linux;
use error::CwaError;

impl_ni_common!(
    runtime_spec_major,
    n_args = 0,
    (_ctx, _args, _mem) => {
        Ok(Some(0))
    }
);

impl_ni_common!(
    runtime_spec_minor,
    n_args = 0,
    (_ctx, _args, _mem) => {
        Ok(Some(0))
    }
);

impl_ni_common!(
    runtime_name,
    n_args = 2,
    (_ctx, args, mem) => {
        static RT_NAME: &'static [u8] = b"Cervus";

        let out_base = args[0] as u32 as usize;
        let out_len = args[1] as u32 as usize;
        let out = mem.checked_slice_mut(out_base, out_base + out_len)?;

        Ok(Some(
            if out.len() < RT_NAME.len() {
                CwaError::InvalidArgument.status() as i64
            } else {
                out[0..RT_NAME.len()].copy_from_slice(RT_NAME);
                RT_NAME.len() as i64
            }
        ))
    }
);

impl_ni_common!(
    runtime_msleep,
    n_args = 1,
    (ctx, args, _mem) => {
        let ms = args[0] as u32;
        let ret = unsafe { linux::lapi_env_msleep(ctx.kctx, ms) };

        if ret < 0 {
            Err(BackendError::FatalSignal)
        } else {
            Ok(None)
        }
    }
);
