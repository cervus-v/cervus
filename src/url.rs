use linux;
use error::*;
use resource::*;
use alloc::boxed::Box;

pub struct Url<'a> {
    pub scheme: &'a str,
    pub path: &'a str,
    pub unparsed_args: &'a str
}

impl<'a> Url<'a> {
    // TODO: scheme registry
    pub fn open(&self, kctx: *mut u8) -> IoResult<Box<Resource>> {
        match self.scheme {
            "file" => {
                // TODO: parse args (flags, etc.)
                static FLAGS: &'static [u8] = b"r";

                let path = self.path.as_bytes();
                if path.len() == 0 {
                    return Err(IoError::Invalid);
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
                    Err(IoError::Generic)
                } else {
                    unsafe { LinuxFile::from_raw_checked(kctx, file, true) }
                        .map(|v| Box::new(v) as Box<Resource>)
                }
            },
            _ => Err(IoError::Invalid)
        }
    }

    pub fn parse(u: &'a str) -> CwaResult<Url<'a>> {
        let mut remaining_slashes: usize = 0;
        let mut scheme_end: Option<usize> = None;
        let mut path_begin: Option<usize> = None;
        let mut path_end: Option<usize> = None;

        for (i, c) in u.chars().enumerate() {
            if remaining_slashes > 0 {
                if c != '/' {
                    return Err(CwaError::InvalidArgument);
                }
                remaining_slashes -= 1;
                continue;
            }

            if scheme_end.is_none() {
                if c == ':' {
                    scheme_end = Some(i);
                    remaining_slashes = 2;
                }
            } else if path_end.is_none() {
                if path_begin.is_none() {
                    path_begin = Some(i);
                }

                if c == '?' {
                    path_end = Some(i);
                }
            } else {
                // args
            }
        }

        let scheme_end = match scheme_end {
            Some(v) => v,
            None => return Err(CwaError::InvalidArgument)
        };
        let path_begin = match path_begin {
            Some(v) => v,
            None => return Err(CwaError::InvalidArgument)
        };
        let path_end = match path_end {
            Some(v) => v,
            None => u.len()
        };

        Ok(Url {
            scheme: &u[0..scheme_end],
            path: &u[path_begin..path_end],
            unparsed_args: &u[path_end..]
        })
    }
}
