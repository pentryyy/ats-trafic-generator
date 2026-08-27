use anyhow::{Context, Result};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub log_level: String,
    pub audio: AudioConfig,
    pub frame: FrameConfig,
    pub server: ServerConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub fft_size: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FrameConfig {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub send_buf: usize,
    pub host: String,
    pub audio_port: u16,
    pub frame_port: u16,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = env::var("CONFIG_PATH")
            .with_context(|| "Переменная окружения CONFIG_PATH не задана")?;

        let data = fs::read_to_string(&config_path)
            .with_context(|| format!("Не удалось прочитать конфиг {:?}", config_path))?;

        let cfg: AppConfig = serde_yaml::from_str(&data)
            .with_context(|| format!("Ошибка парсинга конфига {:?}", config_path))?;

        Ok(cfg)
    }

    pub fn audio_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.audio_port)
    }

    pub fn frame_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.frame_port)
    }

    pub fn send_buf(&self) -> Vec<u8> {
        vec![0u8; self.server.send_buf]
    }

    pub fn log_level(&self) -> LevelFilter {
        match self.log_level.to_lowercase().as_str() {
            "off" => LevelFilter::Off,
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        }
    }
}
