use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
    #[serde(default)]
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub shell: String,
    pub ai_enabled: bool,
    pub max_suggestions: usize,
    pub debounce_ms: u64,
    pub deterministic_first: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            shell: "fish".to_string(),
            ai_enabled: false,
            max_suggestions: 3,
            debounce_ms: 300,
            deterministic_first: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub timeout_ms: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            model: "qwen2.5:0.5b".to_string(),
            endpoint: "http://127.0.0.1:11434".to_string(),
            timeout_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub ghost_text: bool,
    pub popup: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            ghost_text: true,
            popup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub accept: String,
    pub show: String,
    pub dismiss: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            accept: "right".to_string(),
            show: "ctrl-space".to_string(),
            dismiss: "escape".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub warn_destructive: bool,
    pub never_auto_execute: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            warn_destructive: true,
            never_auto_execute: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ai: AiConfig::default(),
            ui: UiConfig::default(),
            keybindings: KeybindingsConfig::default(),
            safety: SafetyConfig::default(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        let new_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/home/prithibi/.config"))
            .join("shellsense");
        if new_dir.exists() {
            return new_dir;
        }
        // Check if legacy config exists
        let legacy = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/home/prithibi/.config"))
            .join("terminal-assistant");
        if legacy.exists() {
            return legacy;
        }
        new_dir
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn data_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/home/prithibi/.local/share"))
            .join("shellsense")
    }

    pub fn socket_path() -> PathBuf {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("shellsense.sock")
        } else {
            let uid = libc_getuid_or_default();
            PathBuf::from(format!("/tmp/shellsense-{}.sock", uid))
        }
    }

    pub fn load_or_default() -> Self {
        let path = Self::config_file();
        if path.exists() {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&contents) {
                    return config;
                }
            }
        }
        let default_config = Self::default();
        let _ = Self::save(&default_config);
        default_config
    }

    pub fn save(config: &Config) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let toml_str = toml::to_string_pretty(config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(Self::config_file(), toml_str)
    }

    pub fn set_ai_enabled(&mut self, enabled: bool) -> std::io::Result<()> {
        self.general.ai_enabled = enabled;
        Self::save(self)
    }

    pub fn set_ai_model(&mut self, model: &str) -> std::io::Result<()> {
        self.ai.model = model.to_string();
        Self::save(self)
    }
}

fn libc_getuid_or_default() -> u32 {
    1000
}
