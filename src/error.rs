use ron::error::SpannedError;

#[derive(thiserror::Error, Debug)]
pub enum DkkError {
    #[error("Not authorized to read file")]
    NotAuthorized,
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Render(#[from] eframe::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    ParseError(#[from] SpannedError),
}

unsafe impl Send for DkkError {}
unsafe impl Sync for DkkError {}
