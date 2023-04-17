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

    pub fn get(&self, mut v: impl FnMut(Option<&T>)) {
        let self_ref = self.0.clone();
        if let Ok(wallet_guard) = self_ref.lock() {
            match &*wallet_guard {
                Some(w) => v(Some(w)),
                _ => v(None),
            };
        };
    }

    pub fn is_set(&self) -> bool {
        let mut is_set = false;
        let self_ref = self.0.clone();
        if let Ok(guard) = self_ref.lock() {
            is_set = match &*guard {
                Some(_) => true,
                _ => false,
            };
        }
        is_set
    }
}
