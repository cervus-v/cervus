use linux;
use error::*;
use resource::*;
use alloc::boxed::Box;
use alloc::BTreeMap;
use alloc::String;

#[derive(Debug)]
pub struct Url<'a> {
    pub scheme: &'a str,
    pub path: &'a str,
    pub unparsed_args: &'a str
}

pub struct SchemeRegistry {
    schemes: BTreeMap<String, Box<SchemeProvider>>
}

impl SchemeRegistry {
    pub fn new() -> SchemeRegistry {
        let mut reg = SchemeRegistry {
            schemes: BTreeMap::new()
        };

        reg.register("file", ::schemes::file::File);
        reg.register("ipc-broadcast", ::schemes::ipc::Broadcast);

        reg
    }

    fn register<S: Into<String>, T: SchemeProvider>(&mut self, key: S, provider: T) {
        self.schemes.insert(key.into(), Box::new(provider));
    }
}

pub trait SchemeProvider: Send + Sync + 'static {
    fn open(&self, url: &Url, kctx: *mut u8) -> KernelResult<IoResult<Box<Resource>>>;
}

impl<'a> Url<'a> {
    // TODO: scheme registry
    pub fn open(&self, kctx: *mut u8) -> KernelResult<IoResult<Box<Resource>>> {
        match ::global::get_global().scheme_registry.schemes.get(self.scheme) {
            Some(provider) => {
                provider.open(self, kctx)
            },
            None => Ok(Err(IoError::Invalid))
        }
    }

    pub fn parse_args(&'a self) -> BTreeMap<&'a str, &'a str> {
        let mut map: BTreeMap<&'a str, &'a str> = BTreeMap::new();
        for mut pair in self.unparsed_args.split("&").map(|v| v.splitn(2, "=")) {
            let k = pair.next().unwrap(); // `split(n)` should always produce at least one element
            let v = match pair.next() {
                Some(v) => v,
                None => ""
            };
            if k.len() > 0 {
                map.insert(k, v);
            }
        }

        map
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
            unparsed_args: if path_end < u.len() {
                // FIXME: this is too hacky
                &u[path_end + 1..] // skip `?`
            } else {
                ""
            }
        })
    }
}
