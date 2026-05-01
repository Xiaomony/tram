use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, HorizontalAlignment, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, List, ListState, Paragraph, Row, Table, TableState},
};
use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::core::{
    btrfs_manager::BtrfsManager,
    btrfs_objects::{group::Group, snapshot_type::SnapshotType},
    error::AppResult,
};
use crate::globals;
use crate::tui::ui_states::Menu;

pub struct AppTUI {
    menu_state: ListState,
    manual_snapshot_table_state: TableState,
    scheduled_snapshot_table_state: TableState,
    btrfs_mgr: Rc<RefCell<BtrfsManager>>,
    /// the index of current selected snapshot group
    selected_group: Option<usize>,
}

impl AppTUI {
    pub fn new(btrfs_mgr: Rc<RefCell<BtrfsManager>>) -> Self {
        Self {
            menu_state: ListState::default().with_selected(Some(0)),
            manual_snapshot_table_state: TableState::default().with_selected(None),
            scheduled_snapshot_table_state: TableState::default().with_selected(None),
            btrfs_mgr,
            selected_group: None,
        }
    }

    /// return the reference of the current selected group
    /// if the index is invalid, try to return the first group
    /// if the group list is empty, return `None`
    fn get_sel_group(&mut self) -> Option<Ref<'_, Group>> {
        let mgr = self.btrfs_mgr.borrow();
        if let Some(index) = self.selected_group
            && index < mgr.get_groups().len()
        {
            Some(Ref::map(mgr, |m| m.get_groups().get(index).unwrap()))
        } else if !mgr.get_groups().is_empty() {
            self.selected_group = Some(0);
            Some(Ref::map(mgr, |m| m.get_groups().first().unwrap()))
        } else {
            None
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(79)])
            .margin(1)
            .spacing(1)
            .split(frame.area());

        // render menu list
        {
            let menu_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" 󰍜 Menu ")
                .title_alignment(HorizontalAlignment::Center);
            let highlight_style = Style::default().fg(Color::Black).bg(Color::LightBlue);
            let list = List::new(globals::MENU_ITEMS)
                .style(Color::Cyan)
                .highlight_style(highlight_style)
                .highlight_spacing(ratatui::widgets::HighlightSpacing::WhenSelected)
                .block(menu_block);
            frame.render_stateful_widget(list, horizontal_layout[0], &mut self.menu_state);
        }

        // render main block
        {
            let crr_menu_item = globals::MENU_ITEMS[self.menu_state.selected().unwrap()];
            use Menu::*;
            match crr_menu_item {
                Snapshots => self.render_snapshots(frame, horizontal_layout[1]),
                Groups => self.render_groups(frame, horizontal_layout[1]),
                Subvolumes => self.render_subvolumes(frame, horizontal_layout[1]),
                BrokenSnapshots => self.render_broken_snapshots(frame, horizontal_layout[1]),
                Settings => self.render_settings(frame, horizontal_layout[1]),
            }
        }
    }

    fn render_snapshots(&mut self, frame: &mut Frame, area: Rect) {
        let mut manual_snapshots = Vec::new();
        let mut scheduled_snapshots = Vec::new();
        // put these logic in a block to make sure the reference `group` drops early
        // otherwise, it will conflict with `&mut self.snapshot_table_state` below
        {
            let Some(group) = self.get_sel_group() else {
                // TODO: no groups
                todo!();
            };
            let snapshots = group.get_snapshots();
            for x in snapshots {
                if x.get_type() == SnapshotType::Manually {
                    manual_snapshots.push(Row::new([x.get_date(), x.get_time()]));
                } else {
                    scheduled_snapshots.push(Row::new([
                        x.get_date(),
                        x.get_time(),
                        x.get_type().to_string(),
                    ]));
                }
            }
        }

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            // TODO: modify the percentage
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        // render manual snapshots block
        {
            let manual_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Manual Snapshots ")
                .title_alignment(HorizontalAlignment::Center);
            if manual_snapshots.is_empty() {
                frame.render_widget(
                    Paragraph::new("No manual snapshots")
                        .alignment(Alignment::Center)
                        .block(manual_block),
                    vertical_layout[0],
                );
            } else {
                let header =
                    Row::new(["Date", "Time"]).style(Style::new().bold().italic().underlined());
                let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];
                let table = Table::new(manual_snapshots, widths)
                    .header(header)
                    .column_spacing(1);
                frame.render_stateful_widget(
                    table.block(manual_block),
                    vertical_layout[0],
                    &mut self.manual_snapshot_table_state,
                );
            };
        }
        // render scheduled snapshots block
        {
            // TEST: these code hasn't been verified and tested yet
            let scheduled_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Scheduled Snapshots ")
                .title_alignment(HorizontalAlignment::Center);
            if scheduled_snapshots.is_empty() {
                frame.render_widget(
                    Paragraph::new("No scheduled snapshots")
                        .alignment(Alignment::Center)
                        .block(scheduled_block),
                    vertical_layout[1],
                );
            } else {
                let header = Row::new(["Date", "Time", "Type"])
                    .style(Style::new().bold().italic().underlined());
                let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];
                let table = Table::new(scheduled_snapshots, widths)
                    .header(header)
                    .column_spacing(1);
                frame.render_stateful_widget(
                    table.block(scheduled_block),
                    vertical_layout[1],
                    &mut self.manual_snapshot_table_state,
                );
            };
        }
    }

    fn render_groups(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(group) = self.get_sel_group() {
            //TODO:
        }
    }
    fn render_subvolumes(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(group) = self.get_sel_group() {
            //TODO:
        }
    }
    fn render_broken_snapshots(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(group) = self.get_sel_group() {
            //TODO:
        }
    }
    fn render_settings(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(group) = self.get_sel_group() {
            //TODO:
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
