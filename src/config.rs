use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::filesystem::DayAndYear;

#[derive(Debug)]
pub struct Config {
    session_token: String,
    cache_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct TomlConf {
    aoc: AocConf,
    cache: CacheConf,
}

#[derive(Debug, Deserialize)]
struct AocConf {
    username: String,
    session: String,
}

#[derive(Debug, Deserialize)]
struct CacheConf {
    dir: PathBuf,
}

impl Config {
    pub fn load_config(config_file: &Path) -> Result<Config, Box<dyn std::error::Error>> {
        let config: TomlConf = toml::from_str(&fs::read_to_string(config_file)?)?;
        let cache_dir = if config.aoc.username.is_empty() {
            config.cache.dir
        } else {
            config.cache.dir.join(&config.aoc.username)
        };

        match fs::create_dir_all(&cache_dir) {
            Ok(()) => (),
            Err(e) => {
                log::error!("Failed to create cache dir ({:?}): {}", cache_dir, e);
                return Err(e.into());
            }
        }

        Ok(Config {
            session_token: config.aoc.session,
            cache_dir,
        })
    }

    #[inline]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    #[inline]
    pub fn cached_day_input(&self, day: DayAndYear) -> PathBuf {
        let mut path = self.cache_dir().to_path_buf();
        path.push(format!("{}", day.year));
        path.push(format!("day{:02}.txt", day.day));

        path
    }

    #[inline]
    pub fn session_token(&self) -> &str {
        &self.session_token
    }
}
