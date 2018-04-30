#[macro_export]
macro_rules! println {
    ($fmt:expr) => ($crate::raw::log(
        $crate::raw::LogLevel::Info,
        &format!($fmt)
    ));
    ($fmt:expr, $($arg:tt)*) => ($crate::raw::log(
        $crate::raw::LogLevel::Info,
        &format!($fmt, $($arg)*)
    ));
}

#[macro_export]
macro_rules! eprintln {
    ($fmt:expr) => ($crate::raw::log(
        $crate::raw::LogLevel::Warning,
        &format!($fmt)
    ));
    ($fmt:expr, $($arg:tt)*) => ($crate::raw::log(
        $crate::raw::LogLevel::Warning,
        &format!($fmt, $($arg)*)
    ));
}

#[macro_export]
macro_rules! main {
    ($body:block) => {
        #[no_mangle]
        pub extern "C" fn __cv_main() {
            { $crate::raw::chk_version(); }
            { $body }
        }
    }
}
