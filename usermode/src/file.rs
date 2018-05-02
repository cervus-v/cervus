use raw;
use std::io::{Read, Write};
use std::io;
use std::cell::RefCell;

thread_local! {
    static STDIN: RefCell<Option<File>> = RefCell::new(File::new_stdin());
    static STDOUT: RefCell<Option<File>> = RefCell::new(File::new_stdout());
    static STDERR: RefCell<Option<File>> = RefCell::new(File::new_stderr());
}

pub struct File {
    fd: i32
}

impl File {
    pub fn new_stdin() -> Option<File> {
        let fd = raw::get_stdin();
        if fd < 0 {
            None
        } else {
            Some(File { fd: fd })
        }
    }

    pub fn new_stdout() -> Option<File> {
        let fd = raw::get_stdout();
        if fd < 0 {
            None
        } else {
            Some(File { fd: fd })
        }
    }

    pub fn new_stderr() -> Option<File> {
        let fd = raw::get_stderr();
        if fd < 0 {
            None
        } else {
            Some(File { fd: fd })
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        raw::close(self.fd);
    }
}

impl Read for File {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let ret = raw::read(self.fd, out);
        if ret < 0 {
            Err(io::Error::from(io::ErrorKind::Other))
        } else {
            Ok(ret as usize)
        }
    }
}

impl Write for File {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let ret = raw::write(self.fd, data);
        if ret < 0 {
            Err(io::Error::from(io::ErrorKind::Other))
        } else {
            Ok(ret as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        // TODO
        Ok(())
    }
}

pub fn with_stdin<F: FnOnce(&mut File) -> T, T>(f: F) -> T {
    STDIN.with(|h| f(h.borrow_mut().as_mut().unwrap()))
}

pub fn with_stdout<F: FnOnce(&mut File) -> T, T>(f: F) -> T {
    STDOUT.with(|h| f(h.borrow_mut().as_mut().unwrap()))
}

pub fn with_stderr<F: FnOnce(&mut File) -> T, T>(f: F) -> T {
    STDERR.with(|h| f(h.borrow_mut().as_mut().unwrap()))
}
