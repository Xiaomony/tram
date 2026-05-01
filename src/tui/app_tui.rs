use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, HorizontalAlignment, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, List, ListState},
};
use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::core::{btrfs_manager::BtrfsManager, btrfs_objects::group::Group, error::AppResult};
use crate::globals;
use crate::tui::menu::Menu;
use crate::tui::snapshots_ui::SnapshotsUI;

#[derive(PartialEq)]
enum AppFocus {
    Menu,
    Body,
    KeyPrompt,
}

pub struct AppTUI {
    snapshot_ui: SnapshotsUI,
    menu_state: ListState,
    _btrfs_mgr: Rc<RefCell<BtrfsManager>>,
    /// the index of current selected snapshot group
    _selected_group: Rc<RefCell<Option<usize>>>,
    focus: AppFocus,
}

impl AppTUI {
    pub fn new(btrfs_mgr: Rc<RefCell<BtrfsManager>>) -> Self {
        let selected_group = Rc::new(RefCell::new(None));
        Self {
            snapshot_ui: SnapshotsUI::new(btrfs_mgr.clone(), selected_group.clone()),
            menu_state: ListState::default().with_selected(Some(0)),
            _btrfs_mgr: btrfs_mgr,
            _selected_group: selected_group,
            focus: AppFocus::Menu,
        }
    }

    fn render_menu(&mut self, frame: &mut Frame, area: Rect) {
        let main_color;
        let highlight_color;
        if self.focus == AppFocus::Menu {
            main_color = globals::FOCUSED_COLOR;
            highlight_color = globals::FOCUSED_HIGHLIHGT_BG_COLOR;
        } else {
            main_color = globals::MENU_COLOR;
            highlight_color = globals::MENU_HIGHLIGHT_BG_COLOR;
        }

        let menu_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" 󰍜 Menu ")
            .title_alignment(HorizontalAlignment::Center);
        let highlight_style = Style::default().fg(Color::Black).bg(highlight_color);
        let list = List::new(globals::MENU_ITEMS)
            .style(main_color)
            .highlight_style(highlight_style)
            .highlight_spacing(ratatui::widgets::HighlightSpacing::WhenSelected)
            .block(menu_block);
        frame.render_stateful_widget(list, area, &mut self.menu_state);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(79)])
            .margin(1)
            .spacing(1)
            .split(frame.area());
        self.render_menu(frame, horizontal_layout[0]);

        // render main block
        let crr_menu_item = globals::MENU_ITEMS[self.menu_state.selected().unwrap()];
        use Menu::*;
        let focused = self.focus == AppFocus::Body;
        match crr_menu_item {
            Snapshots => self
                .snapshot_ui
                .render(frame, horizontal_layout[1], focused),
            Groups => (),
            _ => (),
            // Subvolumes => self.render_subvolumes(frame, horizontal_layout[1]),
            // BrokenSnapshots => self.render_broken_snapshots(frame, horizontal_layout[1]),
            // Settings => self.render_settings(frame, horizontal_layout[1]),
        }
    }

    // returns whether the program should exit
    pub fn read_events(&mut self) -> AppResult<bool> {
        if let Event::Key(key_event) = event::read()? {
            use KeyCode::*;
            let mods = key_event.modifiers;
            match key_event.code {
                Char('k') | Up if mods == KeyModifiers::NONE => self.navigate_up(),
                Char('j') | Down if mods == KeyModifiers::NONE => self.navigate_down(),
                // navigate to top / bottom
                Char('g') | Home => self.navigate_top(),
                Char('G') | End => self.navigate_bottom(),

                Char('q') | Esc => return Ok(true),
                _ => (),
            }
        }
        Ok(false)
    }

    fn navigate_up(&mut self) {
        self.menu_state.select_previous();
    }

    fn navigate_down(&mut self) {
        self.menu_state.select_next();
    }

    fn navigate_bottom(&mut self) {
        self.menu_state.select_last();
    }
    fn navigate_top(&mut self) {
        self.menu_state.select_first();
    }
}

/// return the reference of the current selected group
/// if the index is invalid, try to select and return the first group
/// if the group list is empty, return `None`
pub fn get_sel_group<'a>(
    btrfs_mgr: &'a Rc<RefCell<BtrfsManager>>,
    selected_group: &'a Rc<RefCell<Option<usize>>>,
) -> Option<Ref<'a, Group>> {
    let mgr = btrfs_mgr.borrow();
    if let Some(index) = *selected_group.borrow()
        && index < mgr.get_groups().len()
    {
        Some(Ref::map(mgr, |m| m.get_groups().get(index).unwrap()))
    } else if !mgr.get_groups().is_empty() {
        *selected_group.borrow_mut() = Some(0);
        Some(Ref::map(mgr, |m| m.get_groups().first().unwrap()))
    } else {
        None
    }
}
