macro_rules! declare_stub {
    ($name:ident) => (
        #[no_mangle]
        pub extern "C" fn $name() -> ! {
            ::linux::kernel_panic(stringify!($name))
        }
    )
}

declare_stub!(abort);

declare_stub!(logbf);
declare_stub!(fmax);
declare_stub!(scalbn);
declare_stub!(fmaxf);
declare_stub!(scalbnf);
declare_stub!(logbl);
declare_stub!(scalbnl);
declare_stub!(fmaxl);
declare_stub!(logb);
declare_stub!(__muloti4);
declare_stub!(__floatundidf);
declare_stub!(__floatundisf);
declare_stub!(__udivti3);
