use alloc::arc::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};
use error::*;

pub struct MemoryPressure {
    value: Arc<AtomicUsize>
}

pub struct MemoryPressureHandle {
    value: Arc<AtomicUsize>,
    contrib: AtomicUsize
}

impl MemoryPressure {
    pub fn new() -> MemoryPressure {
        MemoryPressure {
            value: Arc::new(AtomicUsize::new(0))
        }
    }

    pub fn read(&self) -> usize {
        self.value.load(Ordering::SeqCst)
    }

    pub fn handle(&self) -> MemoryPressureHandle {
        MemoryPressureHandle {
            value: self.value.clone(),
            contrib: AtomicUsize::new(0)
        }
    }
}

impl Clone for MemoryPressureHandle {
    fn clone(&self) -> MemoryPressureHandle {
        MemoryPressureHandle {
            value: self.value.clone(),
            contrib: AtomicUsize::new(0)
        }
    }
}

impl Drop for MemoryPressureHandle {
    fn drop(&mut self) {
        self.value.fetch_sub(self.contrib.load(Ordering::Relaxed), Ordering::SeqCst);
    }
}

impl MemoryPressureHandle {
    // FIXME: Overflow?
    pub fn inc(&self, n: usize) {
        self.contrib.fetch_add(n, Ordering::SeqCst);
        self.value.fetch_add(n, Ordering::SeqCst);
    }

    pub fn dec(&self, n: usize) {
        self.contrib.fetch_sub(n, Ordering::SeqCst);
        self.value.fetch_sub(n, Ordering::SeqCst);
    }
}
