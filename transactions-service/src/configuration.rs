use config::Config;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub bucket_name: String,
    pub scope_name: String,
    pub collection_name: String,
    pub test_bucket_name: String,
    pub test_scope_name: String,
    pub test_collection_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!("couchbase://{}:{}", self.host, self.port)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()
        .unwrap();

    settings.try_deserialize()
}
