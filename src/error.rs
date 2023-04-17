#[derive(thiserror::Error, Debug)]
pub enum DaikokuError {
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}
