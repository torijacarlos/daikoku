use std::sync::{Arc, RwLock};

use crate::error::DaikokuError;

pub type DaikokuResult<T> = Result<T, DaikokuError>;
pub type ThreadData<T> = Arc<RwLock<Option<T>>>;

pub struct DaikokuThreadData<T>(pub ThreadData<T>);

impl<T> DaikokuThreadData<T> {
    pub fn empty() -> Self {
        Self(Arc::new(RwLock::new(None)))
    }

    pub fn clone(&self) -> ThreadData<T> {
        self.0.clone()
    }

    pub fn get(&self, mut v: impl FnMut(Option<&T>)) {
        if let Ok(wallet_guard) = self.0.read() {
            match &*wallet_guard {
                Some(ref w) => v(Some(w)),
                _ => v(None),
            };
        }
    }
}
