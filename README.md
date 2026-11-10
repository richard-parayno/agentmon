# agentmon

A macOS menu bar app that puts agent usage and agent status in one place:

- **Usage**: ChatGPT and Claude limits (when configured), and OpenCode Go usage.
- **Status**: what Hermes and Amp agents are doing, and which ones wait on you.

The menu bar shows the fullest usage meter and how many agents wait on you (`71% · 1 waiting`).
Click it for the popover with every provider. Every provider returns **sample data** for now;
each card says so until its source is wired up.

## Run it

Tools come from [mise](https://mise.jdx.dev) (`mise.toml` pins Rust 1.99.1, the toolchain Rune
builds with).

```sh
mise trust          # once, to allow this repo's mise.toml
mise install        # installs the pinned Rust toolchain
mise run run        # build (debug) and run; the item appears in the menu bar
```

The first build fetches and compiles Rune and wgpu, which takes a few minutes.

| Task | What it does |
| --- | --- |
| `mise run run` | Debug build, run in the menu bar |
| `mise run dev` | Release build, run in the menu bar |
| `mise run open` | Build `target/release/Agentmon.app` and launch it with `open` |
| `mise run bundle` | Only build `Agentmon.app` |
| `mise run snap` | Render the popover headless to `target/agentmon.png` (no window) |
| `mise run test` | Tests |
| `mise run lint` | `cargo fmt --check` and clippy |

Quit from the popover's **Quit** button (the app has no Dock icon). To start it at login, add
`Agentmon.app` in System Settings → General → Login Items.

## Configure

Copy `config.example.toml` to `~/.config/agentmon/config.toml` (or set `AGENTMON_CONFIG`):

```toml
refresh_secs = 60

[providers.claude]
enabled = true

[providers.opencode_go]
enabled = false
```

A provider without a table is enabled. Any other key in a provider's table is passed to that
provider (for API key names, endpoints and so on). A file that doesn't parse is reported on
stderr and the defaults are used.

## How it's built

- **UI: [Rune](https://github.com/anomalyco/rune)**, a GPU-rendered Rust UI framework, with
  `rune-kit` components (Progress, Badge, Button, Divider) on its neutral theme. Rune is
  pre-release, so `Cargo.toml` pins a `dev` revision; bump `rev` on both lines to update.
- **The popover** is one Rune window made as a hidden *launcher panel*
  (`WindowOptions::launcher_panel()`): created and drawn at launch while invisible, so a click
  only shows it. It's a non-activating panel: it takes keys without pulling your other windows
  forward, and hides when it loses focus. The process uses the accessory policy (no Dock icon).
- **The menu bar item** is an AppKit `NSStatusItem` created through `objc2`
  (`src/status_item.rs`). Rune's own `Tray` has no macOS backend yet; its docs leave the tray
  to the app and supply the window handles (`AppHandle::set_position`, `show`, `hide`).
- **Refreshing** runs on a background thread: it calls each provider's `fetch` (which may block),
  then lands the result with `AppHandle::update`, which runs on the UI thread and also updates
  the menu bar title. **Refresh** in the popover wakes the thread early.

```
src/
  main.rs            app setup, status item click → popover, refresh thread
  ui.rs              popover state and view, the menu bar summary
  status_item.rs     NSStatusItem (macOS only)
  config.rs          config file
  providers/
    mod.rs           Provider trait, Snapshot/Item types, REGISTRY
    chatgpt.rs  claude.rs  hermes.rs  amp.rs  opencode_go.rs
```

## Adding or wiring up a provider

A provider implements one trait:

```rust
pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;     // the config table name
    fn name(&self) -> &'static str;   // the card title
    fn fetch(&self) -> Result<Snapshot, String>;
}
```

`fetch` returns `Item::Meter`s (usage against a limit) and/or `Item::Agent`s (an agent and its
status). Return `Snapshot { items, sample: false }` once the data is real, so the card drops its
"Sample data" badge; return `Err(message)` to show the error on the card. A new provider is a
new file in `src/providers/` plus one line in `REGISTRY`.

## Checking the UI without a window

Rune apps render headless. `mise run snap` writes the popover to `target/agentmon.png` with every
provider's data; add `--theme light` or `--size 360x900` after `--` to see other states:

```sh
cargo run -- --snap target/tall.png --size 360x900
```
