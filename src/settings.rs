use config::{Config, ConfigError};
use secrecy::{Secret, ExposeSecret};
use serde::Deserialize;
use sqlx::{mysql::MySqlConnectOptions, MySqlPool, Pool, MySql};

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
    pub fn load() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name("configuration/config.base.yml"))
            .add_source(
                config::Environment::with_prefix("DAIKOKU")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        settings.try_deserialize::<Self>()
    }

    pub async fn get_db_conn_pool(&self) -> Result<Pool<MySql>, sqlx::Error> {
        let options = MySqlConnectOptions::new()
            .host(&self.database.host)
            .username(&self.database.user)
            .password(self.database.pass.expose_secret())
            .port(self.database.port)
            .database(&self.database.name);
        MySqlPool::connect_with(options).await
    }
}
