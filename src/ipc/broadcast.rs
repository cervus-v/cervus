use alloc::Vec;
use alloc::VecDeque;
use alloc::boxed::Box;
use alloc::arc::{Arc, Weak};
use alloc::BTreeMap;
use alloc::String;

use mutex::Mutex;
use sync::Semaphore;
use resource::*;
use error::*;
use slab::Slab;
use memory_pressure::MemoryPressureHandle;

use core::sync::atomic::{AtomicUsize, Ordering};

const MAX_PENDING_MESSAGES: usize = 4096;

pub struct Registry {
    channels: Mutex<BTreeMap<String, Weak<BroadcastImpl>>>,
    update_count: AtomicUsize
}

impl Registry {
    pub fn new() -> KernelResult<Registry> {
        Ok(Registry {
            channels: Mutex::new(BTreeMap::new())?,
            update_count: AtomicUsize::new(0)
        })
    }

    pub fn update_gc(&self) -> KernelResult<()> {
        // We don't really need "accurate" counting here since
        // performing a GC is always valid
        if self.update_count.fetch_add(1, Ordering::SeqCst) < 10000 {
            return Ok(());
        }

        self.update_count.store(0, Ordering::Relaxed);

        let mut channels = self.channels.lock()?;
        let mut remove_list: Vec<String> = Vec::new();
        for (k, v) in &*channels {
            if v.upgrade().is_none() {
                remove_list.push(k.clone());
            }
        }

        for k in &remove_list {
            channels.remove(k).unwrap(); // The key always exists so this should never fail
        }

        Ok(())
    }

    pub fn get<K: AsRef<str>>(&self, key: K) -> KernelResult<Option<Arc<BroadcastImpl>>> {
        let key = key.as_ref();

        let mut channels = self.channels.lock()?;
        match channels.get(key) {
            Some(v) => {
                if let Some(v) = v.upgrade() {
                    Ok(Some(v))
                } else {
                    channels.remove(key);
                    Ok(None)
                }
            },
            None => Ok(None)
        }
    }

    pub fn add<K: AsRef<str> + Into<String>>(&self, key: K, bc: Weak<BroadcastImpl>) -> KernelResult<CwaResult<()>> {
        self.update_gc()?;

        let mut channels = self.channels.lock()?;
        if let Some(v) = channels.get(key.as_ref()) {
            if v.upgrade().is_some() {
                return Ok(Err(CwaError::InvalidArgument));
            }
        }
        channels.insert(key.into(), bc);
        Ok(Ok(()))
    }
}

#[derive(Clone)]
pub struct Broadcast {
    pub inner: Arc<BroadcastImpl>
}

pub struct BroadcastImpl {
    subscribers: Mutex<Slab<Weak<SubscriberImpl>>>
}

pub struct Owner {
    bc: Broadcast,
    mp: Option<MemoryPressureHandle>
}

pub struct Subscriber {
    inner: Arc<SubscriberImpl>
}

pub struct SubscriberImpl {
    id: AtomicUsize, // FIXME: we don't actually need atomicity
    bc: Weak<BroadcastImpl>,
    messages: Mutex<VecDeque<Arc<[u8]>>>,
    notify: Semaphore,
    mp: Mutex<Option<MemoryPressureHandle>>
}

impl Resource for Owner {
    fn init_mem_pressure(&mut self, pressure: MemoryPressureHandle) {
        pressure.inc(128);
        self.mp = Some(pressure);
    }

    fn write(&mut self, data: &[u8]) -> KernelResult<IoResult<usize>> {
        let subscribers = self.bc.inner.subscribers.lock()?.clone();
        let data: Arc<[u8]> = Arc::from(data.to_vec().into_boxed_slice());

        subscribers.for_each(|sub| {
            if let Some(sub) = sub.upgrade() {
                let mut messages = sub.messages.lock()?;
                if messages.len() < MAX_PENDING_MESSAGES {
                    messages.push_back(data.clone());
                    sub.notify.up();
                    if let Some(ref mp) = &*sub.mp.lock()? {
                        mp.inc(data.len());
                    } else {
                        println!("Warning: The target subscriber's memory pressure is not initialized. This is a bug.");
                    }
                }
            }

            Ok(())
        })?;

        Ok(Ok(data.len()))
    }

    fn read(&mut self, _out: &mut [u8]) -> KernelResult<IoResult<usize>> {
        Ok(Err(IoError::Invalid))
    }
}

impl Resource for Subscriber {
    fn init_mem_pressure(&mut self, pressure: MemoryPressureHandle) {
        if let Ok(mut mp) = self.inner.mp.lock() {
            if mp.is_some() {
                println!("Warning: Attempting to init memory pressure on a SubscriberImpl multiple times. This is a bug.");
            } else {
                pressure.inc(16);
                *mp = Some(pressure);
            }
        }
    }

    fn write(&mut self, _data: &[u8]) -> KernelResult<IoResult<usize>> {
        Ok(Err(IoError::Invalid))
    }

    fn read(&mut self, out: &mut [u8]) -> KernelResult<IoResult<usize>> {
        // Is the channel closed?
        if self.inner.bc.upgrade().is_none() {
            return Ok(Ok(0));
        }

        // Wait for notification
        self.inner.notify.down()?;
        let msg = if let Some(v) = self.inner.messages.lock()?.pop_front() {
            if let Some(ref mp) = &*self.inner.mp.lock()? {
                mp.dec(v.len());
            }

            v
        } else {
            // Channel closed
            return Ok(Ok(0));
        };

        let copy_len = if msg.len() < out.len() {
            msg.len()
        } else {
            out.len()
        };

        out[0..copy_len].copy_from_slice(&msg[0..copy_len]);

        Ok(Ok(copy_len))
    }
}

impl Broadcast {
    pub fn new() -> KernelResult<(Broadcast, Owner)> {
        let bc = Broadcast {
            inner: Arc::new(BroadcastImpl {
                subscribers: Mutex::new(Slab::new())?
            })
        };
        let owner = Owner { bc: bc.clone(), mp: None };

        Ok((bc, owner))
    }

    pub fn add_to_registry<K: AsRef<str> + Into<String>>(&self, k: K, reg: &Registry) -> KernelResult<CwaResult<()>> {
        reg.add(k, Arc::downgrade(&self.inner))
    }
}

impl BroadcastImpl {
    pub fn subscribe(me: Arc<BroadcastImpl>) -> KernelResult<Subscriber> {
        let sub = Subscriber {
            inner: Arc::new(SubscriberImpl {
                id: AtomicUsize::new(::core::usize::MAX),
                bc: Arc::downgrade(&me),
                messages: Mutex::new(VecDeque::new())?,
                notify: Semaphore::new()?,
                mp: Mutex::new(None)?
            })
        };

        let mut subs = me.subscribers.lock()?;

        let id = subs.insert(Arc::downgrade(&sub.inner));
        sub.inner.id.store(id, Ordering::Relaxed);

        Ok(sub)
    }
}

impl Drop for BroadcastImpl {
    fn drop(&mut self) {
        if let Ok(subs) = self.subscribers.lock() {
            subs.for_each(|sub| -> KernelResult<()> {
                // One more notification to let subscribers know we are not valid any more
                if let Some(sub) = sub.upgrade() {
                    sub.notify.up();
                }
                Ok(())
            }).unwrap_or_else(|_| unreachable!()); // This should never fail since we always return `Ok`
        }
    }
}

impl Drop for SubscriberImpl {
    fn drop(&mut self) {
        if let Some(bc) = self.bc.upgrade() {
            let _ = bc.subscribers.lock().and_then(|mut bc| {
                bc.remove(self.id.load(Ordering::Relaxed)).unwrap_or_else(|_| unreachable!()); // This should never fail
                Ok(())
            });
        }
    }
}
