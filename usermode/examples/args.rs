#![no_main]

#[macro_use]
extern crate usermode;

use std::io::Write;

main!({
    let args = usermode::env::args();
    usermode::file::with_stdout(|out| write!(out, "{:?}\n", args)).unwrap();
});
