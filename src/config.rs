//! The user's config: how often to refresh and which providers are on.
//!
//! Read from `$AGENTMON_CONFIG`, else `~/.config/agentmon/config.toml`. A missing file means the
//! defaults (every provider on); a file that doesn't parse is reported and the defaults are used,
//! so a typo never keeps the menu bar item from showing.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Seconds between refreshes of every enabled provider.
    pub refresh_secs: u64,
    /// Keyed by provider id (`chatgpt`, `claude`, `hermes`, `amp`, `opencode_go`).
    pub providers: BTreeMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    pub enabled: bool,
    /// Everything else in the provider's table, for the provider to read (API key paths,
    /// endpoints, account ids). Stubs ignore it.
    #[serde(flatten)]
    pub settings: toml::Table,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: toml::Table::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_secs: 60,
            providers: BTreeMap::new(),
        }
    }
}

impl Config {
    /// A provider with no table in the file is on.
    pub fn provider(&self, id: &str) -> ProviderConfig {
        self.providers.get(id).cloned().unwrap_or_default()
    }

    pub fn path() -> PathBuf {
        if let Some(p) = std::env::var_os("AGENTMON_CONFIG") {
            return PathBuf::from(p);
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default();
        home.join(".config/agentmon/config.toml")
    }

    pub fn load() -> Config {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(src) => Self::parse(&src).unwrap_or_else(|e| {
                eprintln!(
                    "agentmon: {} doesn't parse, using defaults: {e}",
                    path.display()
                );
                Config::default()
            }),
            Err(_) => Config::default(),
        }
    }

    pub fn parse(src: &str) -> Result<Config, toml::de::Error> {
        toml::from_str(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_example_config_parses_and_unlisted_providers_default_on() {
        let c = Config::parse(include_str!("../config.example.toml")).unwrap();
        assert_eq!(c.refresh_secs, 60);
        assert!(!c.provider("opencode_go").enabled);
        assert!(c.provider("claude").enabled);
        assert!(c.provider("not-in-file").enabled);
    }

    #[test]
    fn provider_settings_keep_their_extra_keys() {
        let c = Config::parse(
            "[providers.claude]\nenabled = false\napi_key_env = \"ANTHROPIC_ADMIN_KEY\"\n",
        )
        .unwrap();
        let claude = c.provider("claude");
        assert!(!claude.enabled);
        assert_eq!(
            claude.settings.get("api_key_env").and_then(|v| v.as_str()),
            Some("ANTHROPIC_ADMIN_KEY")
        );
    }
}
