#![no_main]

#[macro_use]
extern crate usermode;

use std::io::Write;

main!({
    for i in 0..10000 {
        usermode::file::with_stdout(|out| write!(out, "Hello world {}\n", i)).unwrap();
    }
});
