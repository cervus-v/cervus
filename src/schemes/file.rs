use url::{SchemeProvider, Url};
use error::*;
use resource::*;
use linux;
use alloc::boxed::Box;

pub struct File;

impl SchemeProvider for File {
    fn open(&self, url: &Url, kctx: *mut u8) -> KernelResult<IoResult<Box<Resource>>> {
        // TODO: parse args (flags, etc.)
        static FLAGS: &'static [u8] = b"r";

        let path = url.path.as_bytes();
        if path.len() == 0 {
            return Ok(Err(IoError::Invalid));
        }

        let file = unsafe { linux::lapi_env_open_file(
            kctx,
            &path[0],
            path.len(),
            &FLAGS[0],
            FLAGS.len()
        ) };

        if file.is_null() {
            // TODO: accurate error code
            Ok(Err(IoError::Generic))
        } else {
            Ok(unsafe { LinuxFile::from_raw_checked(kctx, file, true) }
                .map(|v| Box::new(v) as Box<Resource>))
        }
    }
}
