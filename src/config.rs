use serde::{Deserialize, Serialize};

use crate::app;
use crate::logf;

const DEFAULT_CONFIG_BYTES: &[u8] = include_bytes!("..\\res\\config.toml");

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub keybinds: Keybinds,
    pub compatibility: Compatibility,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Compatibility {
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Keybinds {
    pub terminate_immediate: Vec<String>,
    pub terminate_click: Vec<String>,
    pub terminate_click_confirm: Vec<String>,
    pub terminate_abort: Vec<String>,
}

/// Reads the [`Config`] structure from the `config.toml` file located in `&ProgramData%/xterminate/config.toml`.
///
/// # Panics
///
/// This methods panics if the toml file cannot be parsed or if the file
/// cannot be written to.
#[must_use]
pub fn load() -> Config {
    let default_config = toml::from_slice::<Config>(DEFAULT_CONFIG_BYTES).unwrap();

    let path = app::config_path();

    let content = match std::fs::read(&path) {
        Ok(v) => v,
        Err(_e) => {
            logf!("WARNING: No config file found, creating a default one");

            if !app::appdata_path().exists() {
                std::fs::create_dir_all(app::appdata_path())
                    .expect("failed to create xterminate program data directory");
            }

            // Create and read the default config
            std::fs::write(&path, DEFAULT_CONFIG_BYTES)
                .expect("failed to write default config file to drive");

            logf!("Config file created");

            DEFAULT_CONFIG_BYTES.to_vec()
        }
    };

    let mut config = toml::from_slice::<Config>(&content).expect("failed to parse config file");

    // Check if the current and new config files are compatible, if not replace the old one.
    if config.compatibility.version_major < default_config.compatibility.version_major
        || config.compatibility.version_minor < default_config.compatibility.version_minor
        || config.compatibility.version_patch < default_config.compatibility.version_patch
    {
        logf!(
            "WARNING: Config file compatibility version mismatch, 
                    replacing old config with updated default config 
                    ({}.{}.{}) => {}.{}.{})",
            config.compatibility.version_major,
            config.compatibility.version_minor,
            config.compatibility.version_patch,
            default_config.compatibility.version_major,
            config.compatibility.version_minor,
            config.compatibility.version_patch
        );

        std::fs::write(&path, DEFAULT_CONFIG_BYTES).expect("failed to overwrite old config file");

        config = default_config;

        logf!("Config replaced");
    }

    logf!("Configuration loaded");
    logf!("Config:\n{config:#?}");

    config
}

/// Saves the specified [`Config`] to disk.
///
/// # Panics
///
/// This method will panic if Serde fails to serialize the [`Config`] structure
/// or if the config file cannot be written to for any reason.
pub fn save(config: &Config) {
    logf!("Writing configuration to disk");

    let path = app::config_path();

    let content = toml::to_string_pretty::<Config>(config).expect("failed to serialize config");

    std::fs::write(path, content).expect("failed to write to config file");

    logf!("Configuration successfully written to disk");
}
