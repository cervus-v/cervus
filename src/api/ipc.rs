use super::*;
use error::CwaError;
use ipc::broadcast;

impl_ni_common!(
    ipc_create_broadcast,
    n_args = 2,
    (ctx, args, mem) => {
        let name_begin = args[0] as u32 as usize;
        let name_len = args[1] as u32 as usize;

        let name = mem.extract_str(name_begin, name_len)?;

        let (bc, owner) = broadcast::Broadcast::new()?;

        let id = ctx.add_resource(Box::new(owner));

        Ok(Some(id as _))
    }
);
