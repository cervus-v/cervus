#![no_main]

#[macro_use]
extern crate usermode;

main!({
    println!("Entering sleep");
    usermode::raw::msleep(5000);
    println!("Exiting from sleep");
});
