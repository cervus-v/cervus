use linux;

impl_ni_common!(
    io_get_stdin,
    n_args = 0,
    (ctx, _args, _mem) => {
        let kctx = ctx.kctx;
        Ok(Some(unsafe {
            ctx.add_raw_linux_file(linux::lapi_env_get_stdin(kctx), false)
        } as i64))
    }
);

impl_ni_common!(
    io_get_stdout,
    n_args = 0,
    (ctx, _args, _mem) => {
        let kctx = ctx.kctx;
        Ok(Some(unsafe {
            ctx.add_raw_linux_file(linux::lapi_env_get_stdout(kctx), false)
        } as i64))
    }
);

impl_ni_common!(
    io_get_stderr,
    n_args = 0,
    (ctx, _args, _mem) => {
        let kctx = ctx.kctx;
        Ok(Some(unsafe {
            ctx.add_raw_linux_file(linux::lapi_env_get_stderr(kctx), false)
        } as i64))
    }
);
