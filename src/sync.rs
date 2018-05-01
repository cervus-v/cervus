use linux;
use linux::RawSemaphore;
use error::*;

pub struct Semaphore {
    holder: RawSemHolder
}

struct RawSemHolder {
    raw: *mut RawSemaphore
}

unsafe impl Send for RawSemHolder {}
unsafe impl Sync for RawSemHolder {}

impl Semaphore {
    pub fn new() -> KernelResult<Semaphore> {
        let raw = unsafe {
            linux::lapi_semaphore_new()
        };

        if raw.is_null() {
            Err(KernelError::NoMem)
        } else {
            Ok(Semaphore {
                holder: RawSemHolder {
                    raw: raw
                }
            })
        }
    }

    pub fn up(&self) {
        unsafe {
            linux::lapi_semaphore_up(self.holder.raw);
        }
    }

    pub fn down(&self) -> KernelResult<()> {
        let ret = unsafe {
            linux::lapi_semaphore_down(self.holder.raw)
        };

        if ret != 0 {
            Err(KernelError::FatalSignal)
        } else {
            Ok(())
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            linux::lapi_semaphore_destroy(self.holder.raw);
        }
    }
}
