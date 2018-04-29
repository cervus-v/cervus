use core::cell::Cell;

use linux;
use linux::BoxedSlice;

use hexagon_e::environment::Environment;
use hexagon_e::tape::Tape;
use hexagon_e::error::*;

pub struct ResourceHolder {
    kctx: *mut u8,
    max_mem: usize,
    max_slots: usize,
    mem: BoxedSlice<u8>,
    slots: BoxedSlice<i64>,
    stack: BoxedSlice<Cell<i64>>,
    call_stack: BoxedSlice<Cell<i64>>
}

pub struct ExecutionEnv<'a> {
    kctx: *mut u8,
    max_mem: usize,
    max_slots: usize,
    mem: &'a mut BoxedSlice<u8>,
    slots: &'a mut BoxedSlice<i64>,
    stack: Tape<'a, Cell<i64>>,
    call_stack: Tape<'a, Cell<i64>>
}

macro_rules! try_option {
    ($v:expr) => {
        match $v {
            Some(v) => v,
            None => return None
        }
    }
}

impl<'a> ExecutionEnv<'a> {
    pub fn new(rh: &'a mut ResourceHolder) -> ExecutionEnv<'a> {
        ExecutionEnv {
            kctx: rh.kctx,
            max_mem: rh.max_mem,
            max_slots: rh.max_slots,
            mem: &mut rh.mem,
            slots: &mut rh.slots,
            stack: Tape::from(&*rh.stack),
            call_stack: Tape::from(&*rh.call_stack)
        }
    }
}

impl<'a> Environment for ExecutionEnv<'a> {
    fn get_memory(&self) -> &[u8] {
        &self.mem
    }

    fn get_memory_mut(&mut self) -> &mut [u8] {
        &mut self.mem
    }

    fn grow_memory(&mut self, len_inc: usize) -> ExecuteResult<()> {
        let new_len = self.mem.len() + len_inc;
        if new_len < self.mem.len() || new_len > self.max_mem {
            return Err(ExecuteError::Generic);
        }

        let mut new_mem = match BoxedSlice::new(|| 0, new_len) {
            Some(v) => v,
            None => return Err(ExecuteError::Generic)
        };

        // new_mem.len() >= self.mem.len() holds here.
        new_mem[0..self.mem.len()].copy_from_slice(&self.mem);
        *self.mem = new_mem;

        Ok(())
    }

    fn get_slots(&self) -> &[i64] {
        &self.slots
    }

    fn get_slots_mut(&mut self) -> &mut [i64] {
        &mut self.slots
    }

    fn reset_slots(&mut self, len: usize) -> ExecuteResult<()> {
        if len > self.max_slots {
            return Err(ExecuteError::Generic);
        }

        *self.slots = match BoxedSlice::new(|| 0, len) {
            Some(v) => v,
            None => return Err(ExecuteError::Generic)
        };

        Ok(())
    }

    fn get_stack(&self) -> &Tape<Cell<i64>> {
        &self.stack
    }

    fn get_call_stack(&self) -> &Tape<Cell<i64>> {
        &self.call_stack
    }

    fn do_native_invoke(&mut self, id: usize) -> ExecuteResult<Option<i64>> {
        match id {
            0 => {
                let args = self.stack.prev_many(3)?;
                let level = args[0].get() as i32;
                let text_base = args[1].get() as u32 as usize;
                let text_len = args[2].get() as u32 as usize;
                let text_end = text_base + text_len;

                if text_base >= self.mem.len() || text_end > self.mem.len() || text_end < text_base {
                    return Err(ExecuteError::Bounds);
                }

                let text = &self.mem[text_base .. text_end];
                if let Err(_) = ::core::str::from_utf8(text) {
                    return Err(ExecuteError::Generic);
                }

                unsafe { linux::lapi_env_log(self.kctx, level, if text.len() == 0 {
                    ::core::ptr::null()
                } else {
                    &text[0]
                }, text.len()) };
                Ok(None)
            },
            _ => Err(ExecuteError::InvalidNativeInvoke)
        }
    }
}

#[derive(Clone)]
pub struct EnvConfig {
    pub kctx: *mut u8,
    pub memory_default_len: usize,
    pub memory_max_len: usize,
    pub max_slots: usize,
    pub stack_len: usize,
    pub call_stack_len: usize
}

impl EnvConfig {
    pub fn is_valid(&self) -> bool {
        if self.memory_default_len == 0 || self.memory_max_len == 0
            || self.max_slots == 0
            || self.stack_len == 0 || self.call_stack_len == 0 {
            false
        } else {
            true
        }
    }
}

impl ResourceHolder {
    pub fn new(config: EnvConfig) -> Option<ResourceHolder> {
        if !config.is_valid() {
            None
        } else {
            Some(ResourceHolder {
                kctx: config.kctx,
                max_mem: config.memory_max_len,
                max_slots: config.max_slots,
                mem: try_option!(BoxedSlice::new(|| 0, config.memory_default_len)),
                slots: try_option!(BoxedSlice::new(|| 0, 0)),
                stack: try_option!(BoxedSlice::new(|| Cell::new(0), config.stack_len)),
                call_stack: try_option!(BoxedSlice::new(|| Cell::new(0), config.call_stack_len))
            })
        }
    }
}
