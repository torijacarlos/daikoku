use std::sync::{Mutex, Arc};

use crate::error::DaikokuError;

pub type DaikokuResult<T> = Result<T, DaikokuError>;
pub type ThreadData<T> = Arc<Mutex<Option<Result<T, DaikokuError>>>>;
