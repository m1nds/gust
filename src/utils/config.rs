use crate::builtin::init::GUST_DIR;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Clone)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitConfig {
    pub user: UserConfig,
}

static INSTANCE: OnceLock<GitConfig> = OnceLock::new();

impl GitConfig {
    pub fn get() -> &'static GitConfig {
        INSTANCE.get_or_init(|| {
            GitConfig::load()
        })
    }

    fn default_config() -> Self {
        GitConfig {
            user: UserConfig {
                name: "Your Name".to_string(),
                email: "you@example.com".to_string(),
            },
        }
    }

    fn load() -> Self {
        let config_path = Path::new(GUST_DIR).join("config");

        let content_result = match File::open(&config_path) {
            Ok(mut file) => {
                let mut data = Vec::new();
                match file.read_to_end(&mut data) {
                    Ok(_) => match String::from_utf8(data) {
                        Ok(s) => Some(s),
                        Err(_) => None,
                    },
                    Err(_) => None,
                }
            }
            Err(_) => None,
        };

        match content_result {
            Some(content) => {
                match toml::from_str::<GitConfig>(&content) {
                    Ok(parsed) => parsed,
                    Err(_) => GitConfig::default_config(),
                }
            }
            None => GitConfig::default_config(),
        }
    }
}
