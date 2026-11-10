//! OpenCode Go subscription usage. Placeholder data for now.
//!
//! To make real: find where OpenCode Go reports its usage against the plan's limit (an account
//! endpoint or the OpenCode CLI), configured in `[providers.opencode_go]`. Off in the example
//! config until that source is known.

use super::{Item, Provider, Snapshot, Unit};
use crate::config::ProviderConfig;

pub struct OpenCodeGo {
    _settings: toml::Table,
}

impl OpenCodeGo {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            _settings: config.settings.clone(),
        }
    }
}

impl Provider for OpenCodeGo {
    fn id(&self) -> &'static str {
        "opencode_go"
    }
    fn name(&self) -> &'static str {
        "OpenCode Go"
    }
    fn fetch(&self) -> Result<Snapshot, String> {
        Ok(Snapshot::sample(vec![Item::meter(
            "Monthly usage",
            4.20,
            10.0,
            Unit::Dollars,
            Some("Dec 1"),
        )]))
    }
}
