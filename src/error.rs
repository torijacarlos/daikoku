#[derive(thiserror::Error, Debug)]
pub enum DaikokuError {
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    RenderError(#[from] eframe::Error),
}

unsafe impl Send for DaikokuError {}
unsafe impl Sync for DaikokuError {}
