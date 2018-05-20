use super::*;

impl_ni_common!(
    log_write,
    n_args = 3,
    (ctx, args, mem) => {
        let level = args[0] as i32;
        let text_base = args[1] as u32 as usize;
        let text_len = args[2] as u32 as usize;
        let text = mem.extract_str(text_base, text_len)?;
        ctx.log(level, text);
        Ok(None)
    }
);
