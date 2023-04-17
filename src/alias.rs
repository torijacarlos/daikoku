use std::sync::{Arc, Mutex};

use crate::error::DaikokuError;

pub type DaikokuResult<T> = Result<T, DaikokuError>;
pub type ThreadData<T> = Arc<Mutex<Option<T>>>;

pub struct DaikokuThreadData<T>(pub ThreadData<T>);

impl<T> DaikokuThreadData<T> {
    pub fn empty() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
    
    pub fn clone(&self) -> Arc<Mutex<Option<T>>> {
        self.0.clone()
    }

    pub fn get(&self, mut v: impl FnMut(&T)) {
        if let Ok(wallet_guard) = self.0.lock() {
            if let Some(ref w) = &*wallet_guard {
                v(w)
            };
        }
    }

    pub fn get_option(&self, mut v: impl FnMut(Option<&T>)) {
        if let Ok(wallet_guard) = self.0.lock() {
            match &*wallet_guard {
                Some(w) => v(Some(w)),
                _ => v(None),
            };
        }
    }
}
