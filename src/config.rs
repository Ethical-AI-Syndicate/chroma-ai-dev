//! Configuration for ChromaAI Dev

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Default, Clone)]
pub struct Config {
    pub brave_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub database_url: Option<String>,
}

static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::default()));

pub fn load_config() {
    // Try to load from environment
    let mut config = Config::default();
    
    if let Ok(key) = std::env::var("BRAVE_API_KEY") {
        config.brave_api_key = Some(key);
    }
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        config.openai_api_key = Some(key);
    }
    if let Ok(url) = std::env::var("DATABASE_URL") {
        config.database_url = Some(url);
    }
    
    *CONFIG.lock().unwrap() = config;
}

pub fn get_config() -> Config {
    CONFIG.lock().unwrap().clone()
}

pub fn set_brave_api_key(key: String) {
    CONFIG.lock().unwrap().brave_api_key = Some(key);
}

pub fn set_openai_api_key(key: String) {
    CONFIG.lock().unwrap().openai_api_key = Some(key);
}

pub fn set_database_url(url: String) {
    CONFIG.lock().unwrap().database_url = Some(url);
}
