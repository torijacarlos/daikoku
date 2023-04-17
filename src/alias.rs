use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use crate::error::DaikokuError;

pub type DaikokuResult<T> = Result<T, DaikokuError>;
pub type ThreadData<T> = Arc<Mutex<Option<T>>>;

pub struct DaikokuThreadData<T>(pub ThreadData<T>);

impl<T> DaikokuThreadData<T> {
    pub fn new(v: T) -> Self {
        Self(Arc::new(Mutex::new(Some(v))))
    }

    pub fn empty() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn clone(&self) -> ThreadData<T> {
        self.0.clone()
    }

    pub fn try_lock(&self) -> Result<MutexGuard<Option<T>>, TryLockError<MutexGuard<Option<T>>>> {
        self.0.try_lock()
    }
}
