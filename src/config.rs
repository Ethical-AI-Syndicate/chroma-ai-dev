//! Configuration for ChromaAI Dev

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Default, Clone)]
pub struct Config {
    pub brave_api_key: Option<String>,
    pub database_url: Option<String>,
    /// LLM provider API keys (provider name -> API key)
    pub llm_api_keys: HashMap<String, String>,
}

impl Config {
    /// Get API key for a specific LLM provider
    pub fn get_llm_api_key(&self, provider: &str) -> Option<&String> {
        self.llm_api_keys.get(provider)
    }
    
    /// Set API key for an LLM provider
    pub fn set_llm_api_key(&mut self, provider: String, key: String) {
        self.llm_api_keys.insert(provider, key);
    }
    
    /// List all configured LLM providers
    pub fn llm_providers(&self) -> Vec<String> {
        self.llm_api_keys.keys().cloned().collect()
    }
}

static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(Config::default()));

pub fn load_config() {
    // Try to load from environment
    let mut config = Config::default();
    
    if let Ok(key) = std::env::var("BRAVE_API_KEY") {
        config.brave_api_key = Some(key);
    }
    if let Ok(url) = std::env::var("DATABASE_URL") {
        config.database_url = Some(url);
    }
    
    // Load LLM provider API keys from environment
    let providers = ["anthropic", "claude", "openai", "google", "gemini", "grok"];
    for provider in providers {
        // Try multiple environment variable naming conventions
        let env_vars = vec![
            format!("{}_API_KEY", provider.to_uppercase()),
            format!("{}_KEY", provider.to_uppercase()),
            format!("{}API_KEY", provider.to_uppercase()),
        ];
        
        for env_var in env_vars {
            if let Ok(key) = std::env::var(&env_var) {
                config.llm_api_keys.insert(provider.to_string(), key);
                break;
            }
        }
    }
    
    *CONFIG.lock().unwrap() = config;
}

pub fn get_config() -> Config {
    CONFIG.lock().unwrap().clone()
}

pub fn set_brave_api_key(key: String) {
    CONFIG.lock().unwrap().brave_api_key = Some(key);
}

pub fn set_database_url(url: String) {
    CONFIG.lock().unwrap().database_url = Some(url);
}

pub fn set_anthropic_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("anthropic".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("claude".to_string(), key_clone);
}

pub fn set_openai_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("openai".to_string(), key);
}

pub fn set_gemini_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("google".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("gemini".to_string(), key_clone);
}

pub fn set_grok_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("grok".to_string(), key);
}
