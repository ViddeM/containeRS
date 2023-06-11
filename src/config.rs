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
}

impl Config {
    pub fn new() -> ConfigResult<Config> {
        dotenv::dotenv().ok();
        Ok(Self {
            database_url: load_env_str("DATABASE_URL")?,
            log_db_statements: load_env_bool("LOG_DB_STATEMENTS")?,
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
