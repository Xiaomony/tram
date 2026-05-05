mod core;
mod globals;
mod tui;

use ratatui::{DefaultTerminal, Frame};
use std::{cell::RefCell, rc::Rc};

use crate::{
    core::{btrfs_manager::BtrfsManager, error::CResult},
    tui::app_tui::AppTUI,
};

fn main() -> CResult<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
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
