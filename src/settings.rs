use config::Config;
use serde::Deserialize;

use crate::{alias::DkkResult, error::DkkError};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub crypt_key: String,
}

impl Settings {
    pub fn load() -> DkkResult<Self> {
        let settings = Config::builder()
            .add_source(config::File::with_name("configuration/config.base.yml"))
            .add_source(
                config::Environment::with_prefix("Dkk")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        settings.try_deserialize::<Self>().map_err(DkkError::Config)
    }
}
