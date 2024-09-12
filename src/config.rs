// config.rs

use std::{env, fs};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Title {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    title: Title,
    base_url: String,
    max_tokens: u32,
    temperature: f32,
    frequency_penalty: f32,
    top_p: f32,
    presence_penalty: f32,
}

impl Config {
    // Default values for the Config struct
    fn default() -> Self {
        Config {
            title: Title {
                name: "robbie".to_string(),
            },
            base_url: "http://98.165.69.213:8080".to_string(),
            frequency_penalty: 0.0,
            max_tokens: 4096,
            presence_penalty: 0.0,
            temperature: 0.7,
            top_p: 1.0,
        }
    }

    // Initialize config from TOML or environment variables
    pub fn init() -> Self {
        let config_str = fs::read_to_string("robbie.toml").unwrap_or_default();
        let mut config: Config = toml::from_str(&config_str).unwrap_or_else(|_| Config::default());

        config.base_url = env::var("ROBBIE_BASE_URL").unwrap_or(config.base_url);
        config.max_tokens = env::var("ROBBIE_MAX_TOKENS").ok().and_then(|s| s.parse().ok()).unwrap_or(config.max_tokens);
        config.temperature = env::var("ROBBIE_TEMPERATURE").ok().and_then(|s| s.parse().ok()).unwrap_or(config.temperature);
        config.frequency_penalty = env::var("ROBBIE_FREQUENCY_PENALTY").ok().and_then(|s| s.parse().ok()).unwrap_or(config.frequency_penalty);
        config.top_p = env::var("ROBBIE_TOP_P").ok().and_then(|s| s.parse().ok()).unwrap_or(config.top_p);
        config.presence_penalty = env::var("ROBBIE_PRESENCE_PENALTY").ok().and_then(|s| s.parse().ok()).unwrap_or(config.presence_penalty);

        config
    }

    // Getter methods for the fields
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }

    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    pub fn frequency_penalty(&self) -> f32 {
        self.frequency_penalty
    }

    pub fn presence_penalty(&self) -> f32 {
        self.presence_penalty
    }

    pub fn top_p(&self) -> f32 {
        self.top_p
    }
}

