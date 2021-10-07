
use config::{Config, ConfigError, File};
use log::LevelFilter;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::collections::HashMap;

pub const DEFAULT_COLOR: u32 = 0xF05B4A;
// pub const DEFAULT_ICON: &str = "https://i.imgur.com/L2FoV6P.png";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub bot: BotSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Deserialize)]
pub struct BotSettings {
    pub token: String,
    pub prefix: String,
    pub application_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct LoggingSettings {
    pub level: LevelFilter,
    pub filters: HashMap<String, LevelFilter>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.set_default("logging.level", "DEBUG")?;
        s.set_default("logging.filters.rustic", "INFO")?;

        s.merge(File::with_name("config.toml")).unwrap();

        s.try_into()
    }
}

static SETTINGS: OnceCell<Settings> = OnceCell::new();

pub fn settings() -> &'static Settings {
    SETTINGS.get().expect("Settings were not initialized")
}

pub fn init() {
    match Settings::new() {
        Ok(settings) => {
            let _ = SETTINGS.set(settings);
        }
        Err(e) => {
            panic!("Failed to parse settings: {}", e);
        }
    }
}
