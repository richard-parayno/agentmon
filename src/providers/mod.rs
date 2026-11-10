//! Sources of usage and status. Each provider is its own module behind [`Provider`]; the UI only
//! sees [`Snapshot`]s, so adding a source is a new module plus one line in [`REGISTRY`].
//!
//! Every provider here returns placeholder data (`Snapshot::sample`) until it's wired to the
//! real source.

mod amp;
mod chatgpt;
mod claude;
mod hermes;
mod opencode_go;

use std::sync::Arc;

use crate::config::{Config, ProviderConfig};

/// One source. `fetch` runs on the refresh thread, never the UI thread, so it may block on the
/// network or a subprocess.
pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn fetch(&self) -> Result<Snapshot, String>;
}

/// What a provider knows right now.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub items: Vec<Item>,
    /// True while the provider returns placeholder data; the UI marks it.
    pub sample: bool,
}

impl Snapshot {
    pub fn sample(items: Vec<Item>) -> Self {
        Self {
            items,
            sample: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// A usage limit: `used` of `limit` (in `unit`), resetting at some point.
    Meter {
        label: String,
        used: f64,
        limit: f64,
        unit: Unit,
        resets: Option<String>,
    },
    /// An agent (or session, or thread) and what it's doing.
    Agent {
        name: String,
        status: AgentStatus,
        detail: String,
    },
}

impl Item {
    pub fn meter(label: &str, used: f64, limit: f64, unit: Unit, resets: Option<&str>) -> Self {
        Item::Meter {
            label: label.into(),
            used,
            limit,
            unit,
            resets: resets.map(Into::into),
        }
    }
    pub fn agent(name: &str, status: AgentStatus, detail: &str) -> Self {
        Item::Agent {
            name: name.into(),
            status,
            detail: detail.into(),
        }
    }
    /// A meter's fill in 0..=1.
    pub fn fraction(&self) -> Option<f64> {
        match self {
            Item::Meter { used, limit, .. } if *limit > 0.0 => Some((used / limit).clamp(0.0, 1.0)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Percent,
    Dollars,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Working,
    NeedsInput,
    Idle,
    Error,
}

impl AgentStatus {
    pub fn label(self) -> &'static str {
        match self {
            AgentStatus::Working => "Working",
            AgentStatus::NeedsInput => "Needs you",
            AgentStatus::Idle => "Idle",
            AgentStatus::Error => "Error",
        }
    }
}

/// Builds a provider from its config table.
type Constructor = fn(&ProviderConfig) -> Arc<dyn Provider>;

/// Every provider the app knows, by config id. Order here is display order.
const REGISTRY: &[(&str, Constructor)] = &[
    ("chatgpt", |c| Arc::new(chatgpt::ChatGpt::new(c))),
    ("claude", |c| Arc::new(claude::Claude::new(c))),
    ("hermes", |c| Arc::new(hermes::Hermes::new(c))),
    ("amp", |c| Arc::new(amp::Amp::new(c))),
    ("opencode_go", |c| Arc::new(opencode_go::OpenCodeGo::new(c))),
];

/// The providers the config turns on, in display order.
pub fn enabled(config: &Config) -> Vec<Arc<dyn Provider>> {
    REGISTRY
        .iter()
        .filter_map(|(id, make)| {
            let c = config.provider(id);
            c.enabled.then(|| make(&c))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_turns_providers_off_and_registry_ids_match_providers() {
        let all = enabled(&Config::default());
        let ids: Vec<_> = all.iter().map(|p| p.id()).collect();
        assert_eq!(ids, REGISTRY.iter().map(|(id, _)| *id).collect::<Vec<_>>());

        let c = Config::parse("[providers.hermes]\nenabled = false\n").unwrap();
        let ids: Vec<_> = enabled(&c).iter().map(|p| p.id()).collect();
        assert_eq!(ids, ["chatgpt", "claude", "amp", "opencode_go"]);
    }

    #[test]
    fn meter_fraction_clamps_and_survives_a_zero_limit() {
        assert_eq!(
            Item::meter("x", 150.0, 100.0, Unit::Percent, None).fraction(),
            Some(1.0)
        );
        assert_eq!(
            Item::meter("x", 5.0, 0.0, Unit::Dollars, None).fraction(),
            None
        );
    }
}
