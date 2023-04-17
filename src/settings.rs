use config::Config;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{mysql::MySqlConnectOptions, MySql, MySqlPool, Pool};

use crate::{alias::DkkResult, error::DkkError};

#[derive(Debug, Deserialize)]
pub struct Settings {
    database: DatabaseSettings,
}

#[derive(Debug, Deserialize)]
struct DatabaseSettings {
    host: String,
    port: u16,
    user: String,
    pass: Secret<String>,
    name: String,
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

        settings
            .try_deserialize::<Self>()
            .map_err(DkkError::ConfigError)
    }

    pub fn get_db_conn_pool(&self) -> Pool<MySql> {
        let options = MySqlConnectOptions::new()
            .host(&self.database.host)
            .username(&self.database.user)
            .password(self.database.pass.expose_secret())
            .port(self.database.port)
            .database(&self.database.name);
        MySqlPool::connect_lazy_with(options)
    }
}
