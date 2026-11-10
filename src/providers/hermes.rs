//! Hermes agent status. Placeholder data for now.
//!
//! To make real: ask the Hermes agent (its gateway or local state) which sessions run and
//! whether any waits on the user, with the address in `[providers.hermes]` of the config.

use super::{AgentStatus, Item, Provider, Snapshot};
use crate::config::ProviderConfig;

pub struct Hermes {
    _settings: toml::Table,
}

impl Hermes {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            _settings: config.settings.clone(),
        }
    }
}

impl Provider for Hermes {
    fn id(&self) -> &'static str {
        "hermes"
    }
    fn name(&self) -> &'static str {
        "Hermes"
    }
    fn fetch(&self) -> Result<Snapshot, String> {
        Ok(Snapshot::sample(vec![
            Item::agent(
                "Inbox triage",
                AgentStatus::Working,
                "Reading 12 new emails",
            ),
            Item::agent(
                "Daily digest",
                AgentStatus::Error,
                "Last run failed: gateway timeout",
            ),
        ]))
    }
}
