#[derive(thiserror::Error, Debug)]
pub enum DkkError {
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    RenderError(#[from] eframe::Error),
}

unsafe impl Send for DkkError {}
unsafe impl Sync for DkkError {}
