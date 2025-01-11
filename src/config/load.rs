use crate::config::RasGBConfig;
use std::fs;
use std::path::Path;
use thiserror::Error;

pub fn read_config_from_env() -> Result<RasGBConfig, ConfigLoadError> {
    let home_dir = fs::canonicalize("~").unwrap_or("~".into());
    let default_path = home_dir.join(".datadance/config.toml");

    let mut env_var_set = true;
    let path = match std::env::var("RASGB_PI_CONFIG") {
        Ok(env_path) => fs::canonicalize(&env_path).unwrap_or(env_path.into()),
        Err(_) => {
            env_var_set = false;
            default_path.clone()
        }
    };

    match read_config(&path) {
        Ok(config) => Ok(config),
        Err(error) => match error {
            ConfigLoadError::FileNotFound { .. } => {
                if !env_var_set {
                    Err(ConfigLoadError::EnvironmentVariableNotSet {
                        default_path: default_path.to_string_lossy().to_string(),
                    })
                } else {
                    Err(error)
                }
            }
            _ => Err(error),
        },
    }
}

fn read_config(file_path: impl AsRef<Path>) -> Result<RasGBConfig, ConfigLoadError> {
    let path = file_path.as_ref();
    let path_exists = path.try_exists().unwrap_or(false);
    if !path_exists {
        return Err(ConfigLoadError::FileNotFound {
            path: path.to_string_lossy().to_string(),
        });
    }

    let config_content = fs::read_to_string(file_path)?;

    // Parse the config content from TOML into the Config struct
    let config = toml::from_str(&config_content)?;

    Ok(config)
}

#[derive(Error, Debug)]
pub enum ConfigLoadError {
    #[error("file was not found at: {path:?}")]
    FileNotFound { path: String },
    #[error("config env var (`RASGB_PI_CONFIG`) not set and file not found at default path: {default_path:?}")]
    EnvironmentVariableNotSet { default_path: String },
    #[error("the config could not be loaded due to malformed formatting")]
    MalformedConfig {
        #[from]
        error: toml::de::Error,
    },
    #[error("the config could not be loaded due to an IO error")]
    IoError {
        #[from]
        error: std::io::Error,
    },
    #[error("the config was invalid: {details}")]
    InvalidConfig { details: String },
}
