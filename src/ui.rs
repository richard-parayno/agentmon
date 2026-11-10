//! The popover's state and view (Rune + rune-kit), and the menu bar title derived from it.

use std::sync::mpsc::Sender;

use rune::{Cx, El, ScrollSpec, Tx, col, el, row, spacer, text};
use rune_kit::{Badge, BadgeVariant, Button, ButtonVariant, ControlSize, Divider, Kit, Progress};

use crate::providers::{AgentStatus, Item, Snapshot, Unit};

pub const SIZE: (f32, f32) = (360.0, 480.0);

pub struct State {
    pub panels: Vec<Panel>,
    pub config_path: String,
    pub refresh_secs: u64,
    /// Asks the refresh thread for a refresh now. None headless, where nothing refreshes.
    pub refresh: Option<Sender<()>>,
}

pub struct Panel {
    pub id: &'static str,
    pub name: &'static str,
    /// None until the first fetch lands.
    pub result: Option<Result<Snapshot, String>>,
}

impl State {
    /// The menu bar title: the fullest usage meter, and how many agents wait on the user.
    pub fn summary(&self) -> String {
        let snapshots = || {
            self.panels
                .iter()
                .filter_map(|p| p.result.as_ref()?.as_ref().ok())
        };
        let fullest = snapshots()
            .flat_map(|s| &s.items)
            .filter_map(Item::fraction)
            .fold(None, |m: Option<f64>, f| Some(m.map_or(f, |m| m.max(f))));
        let waiting = snapshots()
            .flat_map(|s| &s.items)
            .filter(|i| {
                matches!(
                    i,
                    Item::Agent {
                        status: AgentStatus::NeedsInput,
                        ..
                    }
                )
            })
            .count();
        let mut title = match fullest {
            Some(f) => format!("{:.0}%", f * 100.0),
            None => "agentmon".to_owned(),
        };
        if waiting > 0 {
            title.push_str(&format!(" · {waiting} waiting"));
        }
        title
    }
}

pub fn view(s: &State, cx: &mut Cx<State>) -> El {
    let k = Kit::neutral(cx.system_dark());
    let c = &k.c;

    let header = row()
        .key("header")
        .px(14.0)
        .py(10.0)
        .gap(8.0)
        .center_y()
        .child(text("agentmon").font(k.ty.ui_medium).color(c.text_base))
        .child(spacer())
        .child(
            Button::new("Refresh")
                .variant(ButtonVariant::Ghost)
                .control_size(ControlSize::Small)
                .disabled(s.refresh.is_none())
                .on_press(|s: &mut State| {
                    if let Some(tx) = &s.refresh {
                        let _ = tx.send(());
                    }
                })
                .build(&k)
                .key("refresh"),
        );

    let mut cards = col().key("cards").w_full().px(10.0).py(10.0).gap(10.0);
    if s.panels.is_empty() {
        cards = cards.child(
            text("No providers are enabled. Turn some on in the config file below.")
                .font(k.ty.text_small)
                .color(c.text_muted)
                .wrap()
                .w_full(),
        );
    }
    for p in &s.panels {
        cards = cards.child(panel(p, &k).key(p.id));
    }

    let footer = row()
        .key("footer")
        .px(14.0)
        .py(8.0)
        .gap(8.0)
        .center_y()
        .child(
            text(format!("{} · every {}s", s.config_path, s.refresh_secs))
                .font(k.ty.caption())
                .color(c.text_faint)
                .truncate()
                .grow(1.0)
                .shrink(),
        )
        .child(
            Button::new("Quit")
                .variant(ButtonVariant::GhostMuted)
                .control_size(ControlSize::Small)
                .on_press_tx(|_: &mut State, tx: &mut Tx| tx.quit())
                .build(&k)
                .key("quit"),
        );

    let body = col()
        .key("popover")
        .w_full()
        .h_full()
        .fill(c.page)
        .child(header)
        .child(Divider::horizontal().build(&k))
        .child(
            el().key("scroll")
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(cards),
        )
        .child(Divider::horizontal().build(&k))
        .child(footer);
    rune_kit::root(body, Vec::new())
}

fn panel(p: &Panel, k: &Kit) -> El {
    let c = &k.c;
    let mut title = row()
        .key("title")
        .gap(8.0)
        .center_y()
        .child(text(p.name).font(k.ty.body_medium).color(c.text_base))
        .child(spacer());
    if let Some(Ok(snap)) = &p.result
        && snap.sample
    {
        title = title.child(
            Badge::new("Sample data")
                .variant(BadgeVariant::Subtle)
                .build(k),
        );
    }
    let card = col()
        .w_full()
        .px(12.0)
        .py(10.0)
        .gap(10.0)
        .radius(k.radius.md)
        .fill(c.bg_base)
        .border(0.5, c.border_weak)
        .child(title);
    match &p.result {
        None => card.child(text("Loading…").font(k.ty.text_small).color(c.text_muted)),
        Some(Err(e)) => card.child(
            text(e.clone())
                .font(k.ty.text_small)
                .color(c.danger.fg)
                .wrap()
                .w_full(),
        ),
        Some(Ok(snap)) => card.children(
            snap.items
                .iter()
                .enumerate()
                .map(|(i, item)| item_row(item, k).key(format!("item-{i}"))),
        ),
    }
}

fn item_row(item: &Item, k: &Kit) -> El {
    let c = &k.c;
    match item {
        Item::Meter {
            label,
            used,
            limit,
            unit,
            resets,
        } => {
            let amount = match unit {
                Unit::Percent => format!("{used:.0}%"),
                Unit::Dollars => format!("${used:.2} / ${limit:.2}"),
            };
            let detail = match resets {
                Some(r) => format!("{amount} · resets {r}"),
                None => amount,
            };
            col()
                .w_full()
                .gap(4.0)
                .child(
                    Progress::new(item.fraction().map(|f| f as f32))
                        .label(label.clone())
                        .build(k),
                )
                .child(text(detail).font(k.ty.caption()).color(c.text_muted))
        }
        Item::Agent {
            name,
            status,
            detail,
        } => {
            let dot = match status {
                AgentStatus::Working => k.c.info.fg,
                AgentStatus::NeedsInput => k.c.warning.fg,
                AgentStatus::Idle => c.icon_faint,
                AgentStatus::Error => k.c.danger.fg,
            };
            let badge = match status {
                AgentStatus::NeedsInput => Badge::new(status.label()).variant(BadgeVariant::Accent),
                _ => Badge::new(status.label()),
            };
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(el().size(8.0, 8.0).radius(4.0).fill(dot))
                .child(
                    col()
                        .grow(1.0)
                        .shrink()
                        .gap(2.0)
                        .child(
                            text(name.clone())
                                .font(k.ty.text_small_medium)
                                .color(c.text_base)
                                .truncate(),
                        )
                        .child(
                            text(detail.clone())
                                .font(k.ty.caption())
                                .color(c.text_muted)
                                .truncate(),
                        ),
                )
                .child(badge.build(k))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(items: Vec<Item>) -> State {
        let panel = Panel {
            id: "p",
            name: "P",
            result: Some(Ok(Snapshot::sample(items))),
        };
        State {
            panels: vec![panel],
            config_path: String::new(),
            refresh_secs: 60,
            refresh: None,
        }
    }

    #[test]
    fn summary_shows_the_fullest_meter_and_waiting_agents() {
        let s = state(vec![
            Item::meter("a", 30.0, 100.0, Unit::Percent, None),
            Item::meter("b", 9.0, 10.0, Unit::Dollars, None),
            Item::agent("t", AgentStatus::NeedsInput, ""),
            Item::agent("u", AgentStatus::Working, ""),
        ]);
        assert_eq!(s.summary(), "90% · 1 waiting");
    }

    #[test]
    fn summary_without_meters_names_the_app() {
        assert_eq!(
            state(vec![Item::agent("t", AgentStatus::Idle, "")]).summary(),
            "agentmon"
        );
    }
}
