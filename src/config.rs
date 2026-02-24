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
    
    // Load LLM provider API keys from ANY environment variable matching pattern:
    // {PROVIDER}_API_KEY or {PROVIDER}_KEY
    // Examples: OPENAI_API_KEY, ANTHROPIC_KEY, COHERE_API_KEY, AMAZON_BEDROCK_KEY, etc.
    for (key, value) in std::env::vars() {
        let key_upper = key.to_uppercase();
        
        // Look for patterns ending with _API_KEY or _KEY
        if key_upper.ends_with("_API_KEY") || key_upper.ends_with("_KEY") {
            // Extract provider name (remove _API_KEY or _KEY suffix)
            let provider = if key_upper.ends_with("_API_KEY") {
                key_upper[..key_upper.len()-9].to_lowercase()
            } else {
                key_upper[..key_upper.len()-4].to_lowercase()
            };
            
            // Normalize some common aliases
            let provider_normalized = match provider.as_str() {
                "claude" | "anthropic" => "anthropic".to_string(),
                "google" | "gemini" => "google".to_string(),
                _ => provider,
            };
            
            config.llm_api_keys.insert(provider_normalized, value);
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

/// Set API key for an LLM provider (generic)
pub fn set_llm_api_key_generic(provider: String, key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert(provider, key);
}

/// Provider-specific setters for convenience

// OpenAI
pub fn set_openai_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("openai".to_string(), key);
}

// Anthropic / Claude
pub fn set_anthropic_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("anthropic".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("claude".to_string(), key_clone);
}

// Google / Gemini
pub fn set_gemini_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("google".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("gemini".to_string(), key_clone);
}

// xAI / Grok
pub fn set_grok_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("grok".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("xai".to_string(), key_clone);
}

// Cohere
pub fn set_cohere_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("cohere".to_string(), key);
}

// Meta / Llama
pub fn set_meta_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("meta".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("llama".to_string(), key_clone);
}

// Mistral AI
pub fn set_mistral_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("mistral".to_string(), key);
}

// AI21 Labs / Jurassic
pub fn set_ai21_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("ai21".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("jurassic".to_string(), key_clone);
}

// Perplexity
pub fn set_perplexity_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("perplexity".to_string(), key);
}

// DeepSeek
pub fn set_deepseek_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("deepseek".to_string(), key);
}

// OpenCode / Big Pickle
pub fn set_opencode_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("opencode".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("big_pickle".to_string(), key_clone);
}

// Amazon Bedrock
pub fn set_aws_bedrock_key(key: String) {
    let key_clone = key.clone();
    let key_clone2 = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("aws".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("bedrock".to_string(), key_clone);
    CONFIG.lock().unwrap().llm_api_keys.insert("amazon".to_string(), key_clone2);
}

// Azure OpenAI
pub fn set_azure_openai_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("azure".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("azure_openai".to_string(), key_clone);
}

// Hugging Face
pub fn set_huggingface_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("huggingface".to_string(), key);
}

// Together AI
pub fn set_together_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("together".to_string(), key);
}

// Replicate
pub fn set_replicate_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("replicate".to_string(), key);
}

// Stability AI
pub fn set_stability_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("stability".to_string(), key);
}

// Alibaba Cloud / Qwen
pub fn set_alibaba_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("alibaba".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("qwen".to_string(), key_clone);
}

// Baidu / Ernie
pub fn set_baidu_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("baidu".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("ernie".to_string(), key_clone);
}

// Zhipu AI / ChatGLM
pub fn set_zhipu_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("zhipu".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("chatglm".to_string(), key_clone);
}

// Moonshot AI
pub fn set_moonshot_api_key(key: String) {
    CONFIG.lock().unwrap().llm_api_keys.insert("moonshot".to_string(), key);
}

// 01.AI (Yi)
pub fn set_01ai_api_key(key: String) {
    let key_clone = key.clone();
    CONFIG.lock().unwrap().llm_api_keys.insert("01ai".to_string(), key);
    CONFIG.lock().unwrap().llm_api_keys.insert("yi".to_string(), key_clone);
}
