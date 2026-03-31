use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub api: ApiConfig,
    pub behavior: BehaviorConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    pub endpoint: String,
    pub model: String,
    pub timeout: u64,
    /// API path prefix (e.g. "/v1" or "/api/v1")
    pub api_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub auto_fix: bool,
    pub confirm_before_run: bool,
    pub max_history_context: usize,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub color: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8000".to_string(),
            model: "default".to_string(),
            timeout: 120,
            api_path: "/v1".to_string(),
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_fix: true,
            confirm_before_run: true,
            max_history_context: 5,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { color: true }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        path.push("npushell");
        path.push("config.toml");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Warning: failed to parse config at {}: {}", path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("Warning: failed to read config at {}: {}", path.display(), e);
                }
            }
        }
        Self::default()
    }

    pub fn display(&self) {
        println!("npushell configuration");
        println!("======================");
        println!("Config file: {}", Self::config_path().display());
        println!();
        println!("[api]");
        println!("  endpoint = \"{}\"", self.api.endpoint);
        println!("  api_path = \"{}\"", self.api.api_path);
        println!("  model    = \"{}\"", self.api.model);
        println!("  timeout  = {}s", self.api.timeout);
        println!();
        println!("[behavior]");
        println!("  auto_fix             = {}", self.behavior.auto_fix);
        println!("  confirm_before_run   = {}", self.behavior.confirm_before_run);
        println!("  max_history_context  = {}", self.behavior.max_history_context);
        println!();
        println!("[ui]");
        println!("  color = {}", self.ui.color);
    }
}
