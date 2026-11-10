//! Claude (claude.ai / Claude Code) usage limits. Placeholder data for now.
//!
//! To make real: read the session and weekly limits Claude Code shows with `/usage`, using
//! credentials named in `[providers.claude]` of the config. Leave the provider off when the
//! user has no Claude account configured.

use super::{Item, Provider, Snapshot, Unit};
use crate::config::ProviderConfig;

pub struct Claude {
    _settings: toml::Table,
}

impl Claude {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            _settings: config.settings.clone(),
        }
    }
}

impl Provider for Claude {
    fn id(&self) -> &'static str {
        "claude"
    }
    fn name(&self) -> &'static str {
        "Claude"
    }
    fn fetch(&self) -> Result<Snapshot, String> {
        Ok(Snapshot::sample(vec![
            Item::meter(
                "Current session",
                71.0,
                100.0,
                Unit::Percent,
                Some("in 1h 42m"),
            ),
            Item::meter(
                "Weekly (all models)",
                46.0,
                100.0,
                Unit::Percent,
                Some("Thu 18:00"),
            ),
            Item::meter(
                "Weekly (Opus)",
                22.0,
                100.0,
                Unit::Percent,
                Some("Thu 18:00"),
            ),
        ]))
    }
}
