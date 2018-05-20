use super::*;
use error::CwaError;

impl_ni_common!(
    resource_read,
    n_args = 3,
    (ctx, args, mem) => {
        let id = args[0] as u32 as usize;
        let mem_begin = args[1] as u32 as usize;
        let len = args[2] as u32 as usize;

        let out = mem.checked_slice_mut(mem_begin, mem_begin + len)?;
        ctx.resources.get_mut(id)?.read(out)
            .map(|n| Some(n as i64))
            .or_else(|_| Ok(Some(CwaError::Unknown.status() as i64)))
    }
);

impl_ni_common!(
    resource_write,
    n_args = 3,
    (ctx, args, mem) => {
        let id = args[0] as u32 as usize;
        let mem_begin = args[1] as u32 as usize;
        let len = args[2] as u32 as usize;

        let data = mem.checked_slice(mem_begin, mem_begin + len)?;
        ctx.resources.get_mut(id)?.write(data)
            .map(|n| Some(n as i64))
            .or_else(|_| Ok(Some(CwaError::Unknown.status() as i64)))
    }
);

impl_ni_common!(
    resource_open,
    n_args = 2,
    (ctx, args, mem) => {
        let url_base = args[0] as u32 as usize;
        let url_len = args[1] as u32 as usize;

        let u = mem.extract_str(url_base, url_len)?;

        Ok(Some(match ::url::Url::parse(u) {
            Ok(u) => {
                match u.open(ctx.kctx) {
                    Ok(f) => ctx.add_resource(f) as i64,
                    Err(e) => e.status() as i64
                }
            },
            Err(e) => e.status() as i64
        }))
    }
);

impl_ni_common!(
    resource_close,
    n_args = 1,
    (ctx, args, _mem) => {
        let id = args[0] as u32 as usize;
        ctx.remove_resource(id)?;

        Ok(None)
    }
);
