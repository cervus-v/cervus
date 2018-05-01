use alloc::{BTreeMap, String};
use mutex::Mutex;
use error::*;

pub struct Registry {
    services: Mutex<BTreeMap<String, Service>>
}

impl Registry {
    pub fn new() -> KernelResult<Registry> {
        Ok(Registry {
            services: Mutex::new(BTreeMap::new())?
        })
    }
}

pub struct Service {

}
