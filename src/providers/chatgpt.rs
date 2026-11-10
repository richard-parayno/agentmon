//! ChatGPT (official app / Codex) usage limits. Placeholder data for now.
//!
//! To make real: read the account's rate-limit windows (the 5-hour and weekly limits Codex shows
//! with `/status`), using credentials named in `[providers.chatgpt]` of the config.

use super::{Item, Provider, Snapshot, Unit};
use crate::config::ProviderConfig;

pub struct ChatGpt {
    _settings: toml::Table,
}

impl ChatGpt {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            _settings: config.settings.clone(),
        }
    }
}

impl Provider for ChatGpt {
    fn id(&self) -> &'static str {
        "chatgpt"
    }
    fn name(&self) -> &'static str {
        "ChatGPT"
    }
    fn fetch(&self) -> Result<Snapshot, String> {
        Ok(Snapshot::sample(vec![
            Item::meter(
                "5-hour limit",
                34.0,
                100.0,
                Unit::Percent,
                Some("in 2h 10m"),
            ),
            Item::meter(
                "Weekly limit",
                58.0,
                100.0,
                Unit::Percent,
                Some("Mon 09:00"),
            ),
        ]))
    }
}
