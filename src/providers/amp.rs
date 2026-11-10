//! Amp thread status. Placeholder data for now.
//!
//! To make real: list the user's recent Amp threads and their agent state (working, waiting on
//! input, idle, error), with an access token named in `[providers.amp]` of the config.

use super::{AgentStatus, Item, Provider, Snapshot};
use crate::config::ProviderConfig;

pub struct Amp {
    _settings: toml::Table,
}

impl Amp {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            _settings: config.settings.clone(),
        }
    }
}

impl Provider for Amp {
    fn id(&self) -> &'static str {
        "amp"
    }
    fn name(&self) -> &'static str {
        "Amp"
    }
    fn fetch(&self) -> Result<Snapshot, String> {
        Ok(Snapshot::sample(vec![
            Item::agent(
                "Scaffold agentmon",
                AgentStatus::Working,
                "Building with cargo",
            ),
            Item::agent(
                "Fix login redirect",
                AgentStatus::NeedsInput,
                "Approve the migration?",
            ),
            Item::agent("Weekly report", AgentStatus::Idle, "Finished 2h ago"),
        ]))
    }
}
