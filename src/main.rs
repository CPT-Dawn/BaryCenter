// ── Barycenter — Main Entrypoint ─────────────────────────────────────────────
//
// 1. Initialize logging.
// 2. Boot the config (embedded asset → disk → parse).
// 3. Initialize runner modules + search engine + frecency DB.
// 4. Launch the iced layer-shell UI on the active monitor.
// ─────────────────────────────────────────────────────────────────────────────

mod config;
mod frecency;
mod runner;
mod search;
mod ui;

use config::AppConfig;
use frecency::FrecencyDb;
use runner::{app::AppRunner, calc::CalcRunner, shell::ShellRunner, sys::SysRunner, Runner};
use search::SearchEngine;

use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, StartMode};

fn main() -> Result<(), iced_layershell::Error> {
    // ── 1. Logging ───────────────────────────────────────────────────────
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    // ── 2. Configuration boot ────────────────────────────────────────────
    let config = AppConfig::load().unwrap_or_else(|e| {
        log::error!("Config error: {}. Using embedded defaults.", e);
        AppConfig::load_embedded()
    });

    log::info!(
        "Barycenter v{} — {}×{} window",
        env!("CARGO_PKG_VERSION"),
        config.width,
        config.height
    );

    // ── 3. Initialize runners + engine + frecency ────────────────────────
    let runners: Vec<Box<dyn Runner>> = vec![
        Box::new(ShellRunner::new(config.terminal.clone())),
        Box::new(AppRunner::new()),
        Box::new(CalcRunner::new()),
        Box::new(SysRunner::new()),
    ];

    let engine = SearchEngine::new();
    let frecency = FrecencyDb::load(config.frecency_decay);

    // ── 4. Build the layer-shell application ─────────────────────────────
    let width = config.width;
    let height = config.height;

    let (state, init_task) = ui::Barycenter::new(config, runners, engine, frecency);

    iced_layershell::build_pattern::application(ui::namespace, ui::update, ui::view)
        .style(ui::style)
        .subscription(ui::subscription)
        .settings(iced_layershell::build_pattern::MainSettings {
            layer_settings: LayerShellSettings {
                size: Some((width, height)),
                anchor: Anchor::empty(),
                layer: Layer::Overlay,
                exclusive_zone: -1,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                start_mode: StartMode::Active,
                ..Default::default()
            },
            ..Default::default()
        })
        .run_with(move || (state, init_task))
}
