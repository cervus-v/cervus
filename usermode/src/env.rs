use raw;

pub fn args() -> Vec<String> {
    let mut buf: [u8; 4096] = unsafe { ::std::mem::uninitialized() };
    let n_args = raw::get_n_args();

    (0..n_args).map(|i| {
        let n = raw::read_arg(i as u32, &mut buf);
        assert!(n >= 0);
        let n = n as usize;

        println!("n = {}", n);

        ::std::str::from_utf8(&buf[0..n]).unwrap().to_string()
    }).collect()
}
