use url::{SchemeProvider, Url};
use error::*;
use resource::*;
use linux;
use alloc::boxed::Box;
use ipc::broadcast;

pub struct Broadcast;

impl SchemeProvider for Broadcast {
    fn open(&self, url: &Url, _kctx: *mut u8) -> KernelResult<IoResult<Box<Resource>>> {
        let path = url.path;
        let options = url.parse_args();

        println!("Opening broadcast channel (url: {:?}) with options: {:?}", url, options);

        if options.get("new").is_some() {
            let (bc, owner) = broadcast::Broadcast::new()?;
            Ok(match bc.add_to_registry(path, &::global::get_global().broadcast_channel_registry)? {
                Ok(_) => Ok(Box::new(owner)),
                Err(e) => {
                    let _: CwaError = e;
                    Err(IoError::Generic)
                }
            })
        } else {
            let bc = match ::global::get_global().broadcast_channel_registry.get(path)? {
                Some(v) => v,
                None => return Ok(Err(IoError::Invalid))
            };
            let subscriber = broadcast::BroadcastImpl::subscribe(bc)?;
            Ok(Ok(Box::new(subscriber)))
        }
    }
}
