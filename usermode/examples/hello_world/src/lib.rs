extern crate usermode;

#[no_mangle]
pub extern "C" fn run() {
    usermode::raw::log(usermode::raw::LogLevel::Info, "Hello, world!");
}
