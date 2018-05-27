use alloc::{Vec, VecDeque};
use error::*;

pub struct Slab<T> {
    len: usize,
    storage: Vec<Option<T>>,
    release_pool: VecDeque<usize>
}

impl<T: Clone> Clone for Slab<T> {
    fn clone(&self) -> Self {
        Slab {
            len: self.len,
            storage: self.storage.clone(),
            release_pool: self.release_pool.clone()
        }
    }
}

impl<T> Slab<T> {
    pub fn new() -> Slab<T> {
        Slab {
            len: 0,
            storage: Vec::new(),
            release_pool: VecDeque::new()
        }
    }

    pub fn get(&self, id: usize) -> KernelResult<&T> {
        if id >= self.storage.len() {
            Err(KernelError::InvalidResource)
        } else {
            match self.storage[id] {
                Some(ref v) => Ok(v),
                None => Err(KernelError::InvalidResource)
            }
        }
    }

    pub fn get_mut(&mut self, id: usize) -> KernelResult<&mut T> {
        if id >= self.storage.len() {
            Err(KernelError::InvalidResource)
        } else {
            match self.storage[id] {
                Some(ref mut v) => Ok(v),
                None => Err(KernelError::InvalidResource)
            }
        }
    }

    pub fn insert(&mut self, val: T) -> usize {
        self.len += 1;

        if let Some(id) = self.release_pool.pop_front() {
            assert!(self.storage[id].is_none());
            self.storage[id] = Some(val);
            id
        } else {
            let id = self.storage.len();
            self.storage.push(Some(val));
            id
        }
    }

    pub fn remove(&mut self, id: usize) -> KernelResult<T> {
        if id >= self.storage.len() {
            Err(KernelError::InvalidResource)
        } else {
            if let Some(v) = self.storage[id].take() {
                self.release_pool.push_back(id);
                self.len -= 1;
                Ok(v)
            } else {
                Err(KernelError::InvalidResource)
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn for_each<E, F: FnMut(&T) -> Result<(), E>>(&self, mut f: F) -> Result<(), E> {
        for elem in &self.storage {
            if let Some(ref v) = *elem {
                f(v)?;
            }
        }

        Ok(())
    }
}
