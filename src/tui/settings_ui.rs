use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::Modifier,
    text::Text,
    widgets::{Block, BorderType, List, ListState, Padding, Paragraph, Row, Table, TableState},
};
use std::{cell::RefCell, rc::Rc};
use tracing::instrument;

use crate::core::{
    app_config::AutoSnapshotSchedule, btrfs_manager::BtrfsManager, error::CResult, utils,
};
use crate::tui::app_tui::{self, AppEvent};
use crate::tui::menu::Menu;

#[derive(PartialEq, Debug)]
enum SettingsUIFocus {
    SubvolumeList,
    SubvolumeDetail { detail: String },
    Settings,
    Instruction,
}

#[derive(Debug)]
pub struct SettingsUI {
    btrfs_mgr: Rc<RefCell<BtrfsManager>>,
    subvol_list_state: ListState,
    settings_table_state: TableState,
    focus: SettingsUIFocus,
}

impl SettingsUI {
    pub fn new(btrfs_mgr: Rc<RefCell<BtrfsManager>>, is_first_time_launch: bool) -> Self {
        Self {
            btrfs_mgr,
            subvol_list_state: ListState::default().with_selected(Some(0)),
            settings_table_state: TableState::new().with_selected_cell(Some((0, 1))),
            focus: if is_first_time_launch {
                SettingsUIFocus::Instruction
            } else {
                SettingsUIFocus::SubvolumeList
            },
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let [subvol_list_area, settings_area] = area.layout(&Layout::vertical([
            Constraint::Length(self.btrfs_mgr.borrow().get_subvolumes().len() as u16 + 2),
            Constraint::Fill(1),
        ]));

        // Render a list of recognized subvolumes
        self.render_subvol_list(
            frame,
            subvol_list_area.inner(Margin::new(2, 0)),
            focused && self.focus == SettingsUIFocus::SubvolumeList,
        );

        // Render settings
        self.render_settings(
            frame,
            settings_area.inner(Margin::new(2, 0)),
            focused && self.focus == SettingsUIFocus::Settings,
        );

        // Render instruction
        match self.focus {
            SettingsUIFocus::SubvolumeDetail {
                detail: ref content,
            } => {
                app_tui::show_confirm_popup(
                    frame,
                    frame.area(),
                    "Subvolume Detail",
                    Paragraph::new(content.clone()),
                    false,
                    true,
                );
            }
            SettingsUIFocus::Instruction => {
                app_tui::show_confirm_popup(
                    frame,
                    frame.area(),
                    "Instruction",
                    Self::get_instruction_paragraph(),
                    false,
                    true,
                );
            }
            _ => (),
        }
    }

    fn render_subvol_list(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let subvol_list_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" 󰨖 Detected Subvolumes ")
            .title_alignment(Alignment::Center)
            .padding(Padding::horizontal(1));
        let mgr = self.btrfs_mgr.borrow();
        let subvol_list = List::from_iter(mgr.get_subvolumes().iter().map(|x| x.to_string_lossy()))
            .block(subvol_list_block)
            .style(app_tui::get_body_color(focused))
            .highlight_style(if focused {
                Modifier::REVERSED
            } else {
                Modifier::empty()
            });
        frame.render_stateful_widget(subvol_list, area, &mut self.subvol_list_state);
    }

    fn render_settings(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let settings_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Menu::Settings)
            .title_alignment(Alignment::Center)
            .style(app_tui::get_body_color(focused))
            .padding(Padding::uniform(1));
        let mgr = self.btrfs_mgr.borrow();
        let schedule = mgr.get_schedule();
        let raw_rows = [
            ("Instruction".to_string(), "Enter/Space".to_string()),
            (
                "Schedule: Daily Snapshots Maximum".to_string(),
                schedule.daily_max.to_string(),
            ),
            (
                "Schedule: Weekly Snapshots Maximum".to_string(),
                schedule.weekly_max.to_string(),
            ),
            (
                "Schedule: Monthly Snapshots Maximum".to_string(),
                schedule.monthly_max.to_string(),
            ),
            (
                "Schedule: Boot Snapshots Maximum".to_string(),
                schedule.boot_max.to_string(),
            ),
        ];
        let rows: Vec<Row> = raw_rows
            .into_iter()
            .map(|x| {
                Row::new([
                    Text::from(x.0).alignment(Alignment::Center),
                    Text::from(x.1).alignment(Alignment::Center),
                ])
            })
            .collect();
        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .cell_highlight_style(if focused {
            Modifier::REVERSED
        } else {
            Modifier::empty()
        })
        .block(settings_block);
        frame.render_stateful_widget(table, area, &mut self.settings_table_state);
    }

    #[inline]
    fn get_instruction_paragraph() -> Paragraph<'static> {
        Paragraph::new(include_str!("../instruction.txt"))
    }

    #[instrument]
    pub fn handle_events(&mut self, event: AppEvent) -> CResult<bool> {
        use AppEvent::*;
        match self.focus {
            SettingsUIFocus::SubvolumeDetail { .. } => {
                if event == Confirm || event == Escape {
                    self.focus = SettingsUIFocus::SubvolumeList;
                }
                return Ok(false);
            }
            SettingsUIFocus::Instruction => {
                if event == Confirm || event == Escape {
                    self.focus = SettingsUIFocus::Settings;
                }
                return Ok(false);
            }
            SettingsUIFocus::SubvolumeList => match event {
                WindowLeft | Escape | Left => return Ok(true),
                WindowDown => self.focus = SettingsUIFocus::Settings,
                Down | Bottom
                    if let Some(i) = self.subvol_list_state.selected()
                        && i >= self
                            .btrfs_mgr
                            .borrow()
                            .get_subvolumes()
                            .len()
                            .saturating_sub(1) =>
                {
                    self.focus = SettingsUIFocus::Settings
                }
                Up => self.subvol_list_state.select_previous(),
                Down => self.subvol_list_state.select_next(),
                Top => self.subvol_list_state.select_first(),
                Bottom => self.subvol_list_state.select_last(),
                Confirm
                    if let Some(i) = self.subvol_list_state.selected()
                        && let mgr = self.btrfs_mgr.borrow()
                        && let subvols = mgr.get_subvolumes()
                        && !subvols.is_empty()
                        && let Some(x) = subvols.get(i.clamp(0, subvols.len() - 1)) =>
                {
                    let detail = utils::expand_tabs(
                        utils::get_subvol_detail(utils::mount_point_join(x).to_string_lossy()),
                        8,
                    );
                    self.focus = SettingsUIFocus::SubvolumeDetail { detail };
                }
                _ => (),
            },
            SettingsUIFocus::Settings => {
                let f = |schedule: &mut AutoSnapshotSchedule, i: usize, is_sub: bool| {
                    let x = match i {
                        1 => &mut schedule.daily_max,
                        2 => &mut schedule.weekly_max,
                        3 => &mut schedule.monthly_max,
                        4 => &mut schedule.boot_max,
                        _ => return,
                    };
                    if is_sub {
                        *x = x.saturating_sub(1);
                    } else {
                        *x = x.saturating_add(1);
                    }
                };
                match event {
                    WindowLeft | Escape => return Ok(true),
                    WindowUp => self.focus = SettingsUIFocus::SubvolumeList,
                    Up | Upward | Top if let Some(0) = self.settings_table_state.selected() => {
                        self.focus = SettingsUIFocus::SubvolumeList
                    }
                    Up => self.settings_table_state.select_previous(),
                    Down => self.settings_table_state.select_next(),
                    Top | Upward => self.settings_table_state.select_first(),
                    Bottom | Downward => self.settings_table_state.select_last(),
                    Right | Left
                        if let Some(i) = self.settings_table_state.selected()
                            && i > 0 =>
                    {
                        let mut schedule = self.btrfs_mgr.borrow().get_schedule();
                        f(&mut schedule, i, event == Left);
                        self.btrfs_mgr.borrow_mut().change_schedule(schedule)?;
                    }
                    Confirm if let Some(0) = self.settings_table_state.selected() => {
                        self.focus = SettingsUIFocus::Instruction
                    }
                    _ => (),
                }
            }
        }
        Ok(false)
    }

    pub fn get_key_prompt(&self) -> (Vec<(AppEvent, &str)>, bool) {
        use AppEvent::*;
        match self.focus {
            SettingsUIFocus::SubvolumeList => (vec![(Confirm, "Details")], true),
            SettingsUIFocus::Settings if let Some(0) = self.settings_table_state.selected() => {
                (vec![(Confirm, "Instruction")], true)
            }
            SettingsUIFocus::Settings if let Some(1..) = self.settings_table_state.selected() => {
                (vec![(Left, "Reduce"), (Right, "Increase")], true)
            }
            SettingsUIFocus::Instruction | SettingsUIFocus::SubvolumeDetail { .. } => {
                (vec![(Confirm, "Ok"), (Escape, "Ok")], false)
            }
            // WARN: unexpected condition
            SettingsUIFocus::Settings => (vec![], false),
        }
    }
}
