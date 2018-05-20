use error::CwaError;

impl_ni_common!(
    env_get,
    n_args = 4,
    (_ctx, _args, _mem) => {
        Ok(Some(CwaError::NotFound.status() as i64))
    }
);
