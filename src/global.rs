use error::KernelResult;

static mut GLOBAL_CONTEXT: GlobalContext = unsafe { uninitialized() };

pub struct GlobalContext {
    pub total_memory: usize,
    pub system_service_registry: ::system_service::Registry,
    pub native_invoke_registry: ::api::Registry,
    pub scheme_registry: ::url::SchemeRegistry,
    pub broadcast_channel_registry: ::ipc::broadcast::Registry
}

impl GlobalContext {
    fn new() -> KernelResult<GlobalContext> {
        Ok(GlobalContext {
            total_memory: unsafe { ::linux::lapi_get_total_ram_bytes() },
            system_service_registry: ::system_service::Registry::new()?,
            native_invoke_registry: ::api::Registry::new(),
            scheme_registry: ::url::SchemeRegistry::new(),
            broadcast_channel_registry: ::ipc::broadcast::Registry::new()?
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn cervus_global_init() -> i32 {
    let ctx = match GlobalContext::new() {
        Ok(v) => v,
        Err(_) => return -1
    };
    ::core::ptr::write(&mut GLOBAL_CONTEXT, ctx);

    println!("Total memory: {} bytes", get_global().total_memory);
    0
}

#[no_mangle]
pub unsafe extern "C" fn cervus_global_cleanup() {
    ::core::ptr::drop_in_place(&mut GLOBAL_CONTEXT);
}

pub fn get_global() -> &'static GlobalContext {
    unsafe { &GLOBAL_CONTEXT }
}

const unsafe fn uninitialized<T>() -> T {
    #[allow(unions_with_drop_fields)]
    union U<T> {
        none: (),
        some: T,
    }

    U { none: () }.some
}
