use error::KernelResult;

static mut GLOBAL_CONTEXT: GlobalContext = unsafe { uninitialized() };

pub struct GlobalContext {
    pub system_service_registry: ::system_service::Registry
}

impl GlobalContext {
    fn new() -> KernelResult<GlobalContext> {
        Ok(GlobalContext {
            system_service_registry: ::system_service::Registry::new()?
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
