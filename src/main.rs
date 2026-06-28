mod core;
mod globals;
mod tui;

use color_eyre::{config::HookBuilder, eyre::Context};
use ratatui::{DefaultTerminal, Frame};
use std::{cell::RefCell, rc::Rc};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    core::{btrfs_manager::BtrfsManager, error::CResult},
    tui::app_tui::AppTUI,
};

fn main() -> CResult<()> {
    #[cfg(not(debug_assertions))]
    let debug_mode = std::env::var("DEBUG").is_ok_and(|v| v == "1");
    #[cfg(debug_assertions)]
    let debug_mode = true;

    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .init();

    HookBuilder::default()
        .display_env_section(false)
        .capture_span_trace_by_default(debug_mode)
        .display_location_section(debug_mode)
        .install()?;
    let result = process_args();
    if debug_mode {
        result
    } else {
        result.wrap_err(">>> If you intend to report this as a bug, please reproduce the bug by 'DEBUG=1 sudo -E tram' and collect output. <<<").wrap_err("=== Skip this error chain and read sections below first! ===")
    }
}

fn process_args() -> CResult<()> {
    let mut args = std::env::args();

    let mut check_schedule = false;
    let mut partion = None;

    while let Some(value) = args.next() {
        match value.as_str() {
            "--check-schedule" => check_schedule = true,
            "--device" if let Some(device) = args.next() => partion = Some(device),
            _ => (),
        }
    }

    let mut btrfs_manager = if let Some(device) = partion {
        BtrfsManager::new(device)?
    } else {
        BtrfsManager::new_default_partion()?
    };

    if check_schedule {
        btrfs_manager.check_schedule()
    } else {
        let result = run(ratatui::init(), btrfs_manager);
        ratatui::restore();
        result
    }
}

fn run(mut terminal: DefaultTerminal, btrfs_manager: BtrfsManager) -> CResult<()> {
    let mgr = Rc::new(RefCell::new(btrfs_manager));
    let mut tui = AppTUI::new(mgr.clone());

    loop {
        terminal.draw(|frame: &mut Frame| tui.render(frame))?;
        if tui.read_events()? {
            break Ok(());
        }
    }
}
