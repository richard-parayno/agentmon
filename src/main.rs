//! agentmon: one menu bar item for agent usage and status.
//!
//! The process is an accessory app (no Dock icon). It keeps a hidden Rune launcher panel (the
//! popover) and an AppKit status item; clicking the item shows the panel under it, and the
//! panel hides again when it loses focus. A background thread refreshes every enabled provider
//! and lands results in the UI state with `AppHandle::update`.
//!
//! `cargo run -- --snap out.png` renders the popover headless with every provider's data.

mod config;
mod providers;
#[cfg(target_os = "macos")]
mod status_item;
mod ui;

use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rune::{App, AppHandle, AppSpec, WindowOptions};

use config::Config;
use providers::Provider;
use ui::{Panel, State};

/// When the popover last hid itself on losing focus. A click on the status item takes focus
/// from the popover before the click lands, so without this the click would reopen it.
static HIDDEN_AT: Mutex<Option<Instant>> = Mutex::new(None);

fn main() {
    rune::prewarm_gpu();
    let config = Config::load();
    let providers = providers::enabled(&config);
    let headless = rune::is_headless();

    let mut panels: Vec<Panel> = providers
        .iter()
        .map(|p| Panel {
            id: p.id(),
            name: p.name(),
            result: None,
        })
        .collect();
    if headless {
        // Snapshots and scripts see real (stub) data instead of "Loading…".
        for (panel, p) in panels.iter_mut().zip(&providers) {
            panel.result = Some(p.fetch());
        }
    }
    let (refresh_tx, refresh_rx) = mpsc::channel();
    let config_path = Config::path()
        .display()
        .to_string()
        .replace(&std::env::var("HOME").unwrap_or_default(), "~");
    let refresh_secs = config.refresh_secs.max(5);

    let app = App::new(AppSpec::new(
        "agentmon",
        ui::SIZE,
        || unreachable!("state comes from App::init"),
        ui::view,
    ));
    let handle = app.handle();
    let app = app
        .init(move || State {
            panels,
            config_path,
            refresh_secs,
            refresh: (!headless).then_some(refresh_tx),
        })
        .accessory()
        .keep_running()
        // Hidden at launch and kept, so a click shows cached pixels instead of opening a window.
        .configured_options(WindowOptions::launcher_panel().macos(|n| n.corner_radius(12.0)))
        .on_focus(|_, focused, tx| {
            if !focused {
                *HIDDEN_AT.lock().unwrap() = Some(Instant::now());
                tx.close_window();
            }
        });

    let mut start = Some((handle, providers, refresh_rx, refresh_secs));
    app.on_ready(move |_| {
        // Once, after the panel's first frame: the status item, then the first refresh.
        let Some((handle, providers, rx, secs)) = start.take() else {
            return;
        };
        install_status_item(handle.clone());
        spawn_refresh(handle, providers, rx, Duration::from_secs(secs));
    })
    .run();
}

#[cfg(target_os = "macos")]
fn install_status_item(handle: AppHandle<State>) {
    status_item::install("agentmon", move |anchor| {
        if handle.is_visible() {
            handle.hide();
            return;
        }
        let just_hid = HIDDEN_AT
            .lock()
            .unwrap()
            .is_some_and(|t| t.elapsed() < Duration::from_millis(300));
        if !just_hid {
            handle.set_position(anchor.popover_origin(ui::SIZE));
            handle.show();
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn install_status_item(handle: AppHandle<State>) {
    // No menu bar item off macOS yet; show the popover as a plain window.
    handle.open();
}

/// Fetches every provider now, then every `every`, or sooner when the Refresh button sends on
/// the channel. Each result lands in the UI state as it arrives.
fn spawn_refresh(
    handle: AppHandle<State>,
    providers: Vec<Arc<dyn Provider>>,
    rx: Receiver<()>,
    every: Duration,
) {
    std::thread::Builder::new()
        .name("agentmon-refresh".into())
        .spawn(move || {
            loop {
                for p in &providers {
                    let (id, result) = (p.id(), p.fetch());
                    handle.update(move |s, _| {
                        if let Some(panel) = s.panels.iter_mut().find(|panel| panel.id == id) {
                            panel.result = Some(result);
                        }
                        #[cfg(target_os = "macos")]
                        status_item::set_title(&s.summary());
                    });
                }
                match rx.recv_timeout(every) {
                    Ok(()) | Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            }
        })
        .expect("spawn the refresh thread");
}
