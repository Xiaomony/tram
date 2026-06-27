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
    let result;
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1
        && let Some(second) = args.get(1)
    {
        match second.as_str() {
            "--check-schedule" => {
                result = BtrfsManager::new_default_partion().and_then(|mut x| x.check_schedule())
            }
            _ => return Ok(()),
        }
    } else {
        let terminal = ratatui::init();
        result = run(terminal);
        ratatui::restore();
    }
    result
}

fn run(mut terminal: DefaultTerminal) -> CResult<()> {
    let mgr = Rc::new(RefCell::new(BtrfsManager::new_default_partion()?));
    let mut tui = AppTUI::new(mgr.clone());

    loop {
        terminal.draw(|frame: &mut Frame| tui.render(frame))?;
        if tui.read_events()? {
            break Ok(());
        }
    }
}
