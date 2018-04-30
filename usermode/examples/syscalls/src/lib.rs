extern crate usermode;

use usermode::raw::{log, LogLevel};

#[no_mangle]
pub extern "C" fn run() {
    log(LogLevel::Info, "Entering sleep");
    usermode::raw::msleep(30000);
    log(LogLevel::Info, "Exiting from sleep");
    log(LogLevel::Info, "Yielding forever");
    loop {
        usermode::raw::yield_now();
    }
}
