use std::sync::{Arc, Mutex};

use crate::error::DkkError;

pub type DkkResult<T> = Result<T, DkkError>;
pub type ThreadData<T> = Arc<Mutex<Option<T>>>;

pub struct DkkThreadData<T>(pub ThreadData<T>);

impl<T> DkkThreadData<T> {
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
            is_set = matches!(&*guard, Some(_));
        }
        is_set
    }
}
