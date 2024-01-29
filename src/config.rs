use std::env::{self, VarError};

#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("Environment variable error")]
    EnvVarError(#[from] VarError),
    #[error("Empty variable error `{0}`")]
    VarEmpty(String),
    #[error("Invalid bool `{0}`")]
    InvalidBool(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;

pub struct Config {
    pub database_url: String,
    pub log_db_statements: bool,
    pub storage_directory: String,
    pub docker_socket_url: String,
    pub registry_url: String,
    pub auth_service: String,
    pub accounts_rs_auth_endpoint: String,
    pub accounts_rs_me_endpoint: String,
}

impl Config {
    pub fn new() -> ConfigResult<Config> {
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: load_env_str("DATABASE_URL")?,
            log_db_statements: load_env_bool("LOG_DB_STATEMENTS")?,
            storage_directory: load_env_str("STORAGE_DIRECTORY")?,
            docker_socket_url: load_env_str("DOCKER_SOCKET_URL")?,
            registry_url: load_env_str("REGISTRY_URL")?,
            auth_service: load_env_str("AUTH_SERVICE")?,
            accounts_rs_auth_endpoint: load_env_str("ACCOUNTS_RS_AUTH_ENDPOINT")?,
            accounts_rs_me_endpoint: load_env_str("ACCOUNTS_RS_ME_ENDPOINT")?,
        })
    }
}

fn load_env_str(key: &str) -> ConfigResult<String> {
    let key = key.to_string();
    let var = env::var(&key)?;

    if var.is_empty() {
        return Err(ConfigError::VarEmpty(key));
    }

    Ok(var)
}

fn load_env_bool(key: &str) -> ConfigResult<bool> {
    let var = load_env_str(key)?;
    match var.as_str() {
        "false" => Ok(false),
        "true" => Ok(true),
        _ => Err(ConfigError::InvalidBool(var)),
    }
}
