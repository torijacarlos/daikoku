use config::{Config, ConfigError};
use serde::Deserialize;
use sqlx::{mysql::MySqlConnectOptions, ConnectOptions, MySqlConnection};

#[derive(Debug, Deserialize)]
pub struct Settings {
    database: DatabaseSettings,
}

#[derive(Debug, Deserialize)]
struct DatabaseSettings {
    host: String,
    port: u16,
    user: String,
    pass: String,
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

        Ok(settings.try_deserialize::<Self>()?)
    }

    pub async fn get_db_conn(&self) -> Result<MySqlConnection, sqlx::Error> {
        MySqlConnectOptions::new()
            .host(&self.database.host)
            .username(&self.database.user)
            .password(&self.database.pass)
            .port(self.database.port)
            .database(&self.database.name)
            .connect()
            .await
    }
}
