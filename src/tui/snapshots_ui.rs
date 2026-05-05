use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, HorizontalAlignment, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Paragraph, Row, Table, TableState},
};
use std::{cell::RefCell, rc::Rc};

use crate::tui::app_tui::{AppEvent, get_sel_group, get_sel_group_mut};
use crate::{
    core::{btrfs_manager::BtrfsManager, btrfs_objects::snapshot_type::SnapshotType},
    globals,
};

#[derive(PartialEq)]
enum SnapshotFocus {
    ManualSnapshot,
    ScheduledSnapshot,
    DetailedInfo,
}

pub struct SnapshotsUI {
    btrfs_mgr: Rc<RefCell<BtrfsManager>>,
    manual_snapshot_table_state: TableState,
    scheduled_snapshot_table_state: TableState,
    /// the index of current selected snapshot group
    selected_group: Rc<RefCell<Option<usize>>>,
    focus: SnapshotFocus,
    manual_snapshot_infos: Vec<(usize, [String; 3])>,
    scheduled_snapshot_infos: Vec<(usize, [String; 4])>,
    no_valid_group: bool,
}

impl SnapshotsUI {
    pub fn new(
        btrfs_mgr: Rc<RefCell<BtrfsManager>>,
        selected_group: Rc<RefCell<Option<usize>>>,
    ) -> Self {
        let mut new_obj = Self {
            btrfs_mgr,
            manual_snapshot_table_state: TableState::default().with_selected(None),
            scheduled_snapshot_table_state: TableState::default().with_selected(None),
            selected_group,
            focus: SnapshotFocus::ManualSnapshot,
            manual_snapshot_infos: Vec::new(),
            scheduled_snapshot_infos: Vec::new(),
            no_valid_group: false,
        };
        new_obj.refresh_table_data();
        new_obj
    }

    pub fn refresh_table_data(&mut self) {
        self.manual_snapshot_infos.clear();
        self.scheduled_snapshot_infos.clear();
        let Some(group) = get_sel_group(&self.btrfs_mgr, &self.selected_group) else {
            self.no_valid_group = true;
            return;
        };
        self.no_valid_group = false;
        let snapshots = group.get_snapshots();
        for (i, x) in snapshots.iter().enumerate() {
            let subvols = x.get_snapshoted_subvolumes().join("  ");
            if x.get_type() == SnapshotType::Manually {
                self.manual_snapshot_infos
                    .push((i, [x.get_date(), x.get_time(), subvols]));
            } else {
                self.scheduled_snapshot_infos.push((
                    i,
                    [
                        x.get_date(),
                        x.get_time(),
                        x.get_type().to_string(),
                        subvols,
                    ],
                ));
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        if self.no_valid_group {
            // TODO: no valid groups
            todo!()
        }
        let manual_snapshot_rows: Vec<Row<'_>> = self
            .manual_snapshot_infos
            .iter()
            .map(|x| Row::new(x.1.clone()))
            .collect();
        let scheduled_snapshot_rows: Vec<Row<'_>> = self
            .scheduled_snapshot_infos
            .iter()
            .map(|x| Row::new(x.1.clone()))
            .collect();

        // determine the height of each block dynamically
        let l1 = manual_snapshot_rows.len();
        let l2 = scheduled_snapshot_rows.len();
        let manual_percentage = if l1 == 0 && l2 == 0 {
            50
        } else {
            ((l1 * 100 / (l1 + l2)) as u16).clamp(20, 80)
        };
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(manual_percentage),
                Constraint::Percentage(100 - manual_percentage),
            ])
            .split(area);

        self.render_manual_block(
            frame,
            vertical_layout[0],
            manual_snapshot_rows,
            focused && self.focus == SnapshotFocus::ManualSnapshot,
        );
        self.render_scheduled_block(
            frame,
            vertical_layout[1],
            scheduled_snapshot_rows,
            focused && self.focus == SnapshotFocus::ScheduledSnapshot,
        );
    }

    #[inline]
    fn get_color(focused: bool) -> Color {
        if focused {
            globals::FOCUSED_COLOR
        } else {
            globals::BODY_COLOR
        }
    }

    fn render_manual_block(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        rows: Vec<Row>,
        focused: bool,
    ) {
        // check focus state and reset table state if it's not focused
        if !focused {
            self.manual_snapshot_table_state.select(None);
        } else if self.manual_snapshot_table_state.selected().is_none() {
            self.manual_snapshot_table_state.select_first();
        }
        let main_color = Self::get_color(focused);
        let manual_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .style(main_color)
            .title(" Manual Snapshots ")
            .title_alignment(HorizontalAlignment::Center);
        if rows.is_empty() {
            frame.render_widget(
                Paragraph::new("No manual snapshots")
                    .alignment(Alignment::Center)
                    .style(globals::WARNING_COLOR)
                    .block(manual_block),
                area,
            );
        } else {
            let header = Row::new(["Date", "Time", "Contained Subvolumes"])
                .style(Style::new().bold().italic().underlined());
            let widths = [
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(40),
            ];
            let table = Table::new(rows, widths)
                .header(header)
                .column_spacing(1)
                .row_highlight_style(Modifier::REVERSED)
                .style(main_color);
            frame.render_stateful_widget(
                table.block(manual_block),
                area,
                &mut self.manual_snapshot_table_state,
            );
        };
    }

    fn render_scheduled_block(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        rows: Vec<Row>,
        focused: bool,
    ) {
        // TEST: these code hasn't been verified and tested yet

        // check focus state and reset table state if it's not focused
        if !focused {
            self.scheduled_snapshot_table_state.select(None);
        } else if self.scheduled_snapshot_table_state.selected().is_none() {
            self.scheduled_snapshot_table_state.select_first();
        }
        let main_color = Self::get_color(focused);
        let scheduled_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .style(main_color)
            .title(" Scheduled Snapshots ")
            .title_alignment(HorizontalAlignment::Center);
        if rows.is_empty() {
            frame.render_widget(
                Paragraph::new("No scheduled snapshots")
                    .alignment(Alignment::Center)
                    .style(globals::WARNING_COLOR)
                    .block(scheduled_block),
                area,
            );
        } else {
            let header = Row::new(["Date", "Time", "Type", "Contained Subvolumes"])
                .style(Style::new().bold().italic().underlined());
            let widths = [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ];
            let table = Table::new(rows, widths)
                .header(header)
                .column_spacing(1)
                .row_highlight_style(Modifier::REVERSED)
                .style(main_color);
            frame.render_stateful_widget(
                table.block(scheduled_block),
                area,
                &mut self.scheduled_snapshot_table_state,
            );
        };
    }

    /// returns whether the focus should be returned to menu
    pub fn handle_events(&mut self, event: AppEvent) -> bool {
        let table_state = if self.focus == SnapshotFocus::ManualSnapshot {
            &mut self.manual_snapshot_table_state
        } else {
            &mut self.scheduled_snapshot_table_state
        };
        use AppEvent::*;
        match event {
            Left | WindowLeft => return true,
            Return => {
                if self.focus == SnapshotFocus::DetailedInfo {
                    self.focus = SnapshotFocus::ManualSnapshot;
                } else {
                    return true;
                }
            }
            Up => table_state.select_previous(),
            Down => table_state.select_next(),
            Top => table_state.select_first(),
            Bottom => table_state.select_last(),
            WindowUp if self.focus != SnapshotFocus::DetailedInfo => {
                self.focus = SnapshotFocus::ManualSnapshot
            }
            WindowDown if self.focus != SnapshotFocus::DetailedInfo => {
                self.focus = SnapshotFocus::ScheduledSnapshot
            }
            Create => {
                if let Some(mut group) = get_sel_group_mut(&self.btrfs_mgr, &self.selected_group) {
                    // TODO: error handling
                    group.create_snapshot(SnapshotType::Manually).unwrap();
                }
                self.refresh_table_data();
            }
            Delete => {
                if let Some(mut group) = get_sel_group_mut(&self.btrfs_mgr, &self.selected_group) {
                    // TODO: error handling
                    if self.focus == SnapshotFocus::ManualSnapshot
                        && let Some(i) = self.manual_snapshot_table_state.selected()
                    {
                        let j = self
                            .manual_snapshot_infos
                            .get(i.clamp(0, self.manual_snapshot_infos.len() - 1))
                            .unwrap()
                            .0;
                        // TODO: error handling
                        group.delete_snapshot(j).unwrap();
                    } else if self.focus == SnapshotFocus::ScheduledSnapshot
                        && let Some(i) = self.scheduled_snapshot_table_state.selected()
                    {
                        let j = self
                            .scheduled_snapshot_infos
                            .get(i.clamp(0, self.manual_snapshot_infos.len() - 1))
                            .unwrap()
                            .0;
                        // TODO: error handling
                        group.delete_snapshot(j).unwrap();
                    }
                }
                self.refresh_table_data();
            }
            Confirm => todo!(),
            // Upward => todo!(),
            // Downward => todo!(),
            _ => (),
        }
        false
    }
}
