use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub registry_url: String,
    pub registry_token: String,
    pub identity_file: PathBuf,
    pub max_file_bytes: u64,
    pub max_request_bytes: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self { registry_url: "https://registry.example.ntp".into(), registry_token: "".into(),
            identity_file: PathBuf::from(".ntps/identity.key"), max_file_bytes: 536_870_912,
            max_request_bytes: 16_384 }
    }
}
impl Config {
    pub fn default_with_registry(registry: Option<String>) -> Self {
        let mut c = Self::default();
        if let Some(v) = registry { c.registry_url = v.trim_end_matches('/').to_string(); }
        c
    }
}
pub fn default_path() -> Result<PathBuf> {
    Ok(std::env::current_dir()?.join("ntps.toml"))
}
pub fn load_default() -> Result<Config> { load(&default_path()?) }
pub fn load(path: &Path) -> Result<Config> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut c: Config = toml::from_str(&raw)?;
    if let Ok(v) = std::env::var("NTPS_REGISTRY_TOKEN") { c.registry_token = v; }
    Ok(c)
}
