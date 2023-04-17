#[derive(thiserror::Error, Debug)]
pub enum DkkError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Render(#[from] eframe::Error),
}

unsafe impl Send for DkkError {}
unsafe impl Sync for DkkError {}
