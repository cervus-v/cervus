use backend::common::*;

use core::cell::Cell;
use alloc::Vec;

use hexagon_e;
use hexagon_e::environment::Environment;
use hexagon_e::tape::Tape;
use hexagon_e::error::*;

impl From<ExecuteError> for BackendError {
    fn from(other: ExecuteError) -> BackendError {
        match other {
            ExecuteError::Generic => BackendError::Generic,
            ExecuteError::Bounds => BackendError::Bounds,
            ExecuteError::InvalidNativeInvoke => BackendError::InvalidNativeInvoke,
            _ => BackendError::Generic
        }
    }
}

impl From<BackendError> for ExecuteError {
    fn from(other: BackendError) -> ExecuteError {
        match other {
            BackendError::Generic => ExecuteError::Generic,
            BackendError::Bounds => ExecuteError::Bounds,
            BackendError::InvalidNativeInvoke => ExecuteError::InvalidNativeInvoke,
            _ => ExecuteError::Generic
        }
    }
}

pub struct ResourceHolder {
    max_mem: usize,
    max_slots: usize,
    mem: Vec<u8>,
    slots: Vec<i64>,
    stack: Vec<Cell<i64>>,
    call_stack: Vec<Cell<i64>>
}

pub struct ExecutionEnv<'a, C: Context + 'a> {
    max_mem: usize,
    max_slots: usize,

    resched_counter: Cell<usize>,

    mem: &'a mut Vec<u8>,
    slots: &'a mut Vec<i64>,
    stack: Tape<'a, Cell<i64>>,
    call_stack: Tape<'a, Cell<i64>>,

    context: &'a mut C
}

impl<'a, C: Context + 'a> ExecutionEnv<'a, C> {
    pub fn new(rh: &'a mut ResourceHolder, ctx: &'a mut C) -> ExecutionEnv<'a, C> {
        ExecutionEnv {
            max_mem: rh.max_mem,
            max_slots: rh.max_slots,

            resched_counter: Cell::new(0),

            mem: &mut rh.mem,
            slots: &mut rh.slots,
            stack: Tape::from(&*rh.stack),
            call_stack: Tape::from(&*rh.call_stack),

            context: ctx
        }
    }
}

impl<'a, C: Context + 'a> Environment for ExecutionEnv<'a, C> {
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

        self.mem.resize(new_len, 0);
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

        *self.slots = vec! [ 0; len ];
        Ok(())
    }

    fn get_stack(&self) -> &Tape<Cell<i64>> {
        &self.stack
    }

    fn get_call_stack(&self) -> &Tape<Cell<i64>> {
        &self.call_stack
    }

    fn do_native_invoke(&mut self, id: usize) -> ExecuteResult<Option<i64>> {
        let policy = self.context.get_native_invoke_policy(id)?;

        let args: &[Cell<i64>] = self.stack.prev_many(policy.n_args)?;

        assert_eq!(::core::mem::size_of::<Cell<i64>>(), ::core::mem::size_of::<i64>());
        let args = unsafe {
            ::core::mem::transmute::<&[Cell<i64>], &[i64]>(args)
        };

        Ok(self.context.do_native_invoke(id, args, &mut self.mem)?)
    }

    #[inline]
    fn trace_opcode(&self, _: &::hexagon_e::module::Opcode) -> ExecuteResult<()> {
        let count = self.resched_counter.get();

        if count >= 100000 {
            self.resched_counter.set(0);
            self.context.tick()?;
        } else {
            self.resched_counter.set(count + 1);
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct EnvConfig {
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
                max_mem: config.memory_max_len,
                max_slots: config.max_slots,
                mem: vec! [ 0; config.memory_default_len ],
                slots: vec! [],
                stack: vec! [ Cell::new(0); config.stack_len ],
                call_stack: vec! [ Cell::new(0); config.call_stack_len ]
            })
        }
    }
}

pub struct HexagonEBackend {
    rh: ResourceHolder
}

impl Backend for HexagonEBackend {
    type Config = EnvConfig;

    fn new(config: EnvConfig) -> BackendResult<HexagonEBackend> {
        Ok(HexagonEBackend {
            rh: match ResourceHolder::new(config) {
                Some(v) => v,
                None => return Err(BackendError::InvalidInput)
            }
        })
    }

    fn run<C: Context>(&mut self, code: &[u8], context: &mut C) -> BackendResult<()> {
        let m = hexagon_e::module::Module::from_raw(code)?;
        let env = ExecutionEnv::new(&mut self.rh, context);

        let mut vm = hexagon_e::vm::VirtualMachine::new(&m, env);
        vm.run_memory_initializers()?;
        vm.run()?;

        Ok(())
    }
}
