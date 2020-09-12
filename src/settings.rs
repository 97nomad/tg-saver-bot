use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct TelegramConfig {
    pub token: String,
    pub allowed_usernames: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadConfig {
    pub target_dir: String,
    pub sticker_tags: Vec<String>,
    pub image_tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub telegram: TelegramConfig,
    pub download: DownloadConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::with_name("config.toml"))?;

        s.try_into()
    }
}
