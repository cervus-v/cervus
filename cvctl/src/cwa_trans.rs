use std::collections::HashMap;

use wasm_core::hetrans::MapNativeInvoke;
use service::ServiceContext;

pub struct Mapper<'a> {
    ctx: &'a ServiceContext,
    cache: HashMap<String, u32>
}

impl<'a> Mapper<'a> {
    pub fn new(ctx: &'a ServiceContext) -> Mapper<'a> {
        Mapper {
            ctx: ctx,
            cache: HashMap::new()
        }
    }
}

impl<'a> MapNativeInvoke for Mapper<'a> {
    fn map_native_invoke(&mut self, module: &str, field: &str) -> Option<u32> {
        if module != "cwa" {
            return None;
        }

        if let Some(v) = self.cache.get(field) {
            return Some(*v);
        }

        if let Some(v) = self.ctx.map_cwa_api(field) {
            self.cache.insert(field.to_string(), v);
            Some(v)
        } else {
            None
        }
    }
}
