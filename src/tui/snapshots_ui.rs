use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, HorizontalAlignment, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph, Row, Table, TableState},
};
use std::{cell::RefCell, rc::Rc};

use crate::tui::app_tui::get_sel_group;
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
}

impl SnapshotsUI {
    pub fn new(
        btrfs_mgr: Rc<RefCell<BtrfsManager>>,
        selected_group: Rc<RefCell<Option<usize>>>,
    ) -> Self {
        Self {
            btrfs_mgr,
            manual_snapshot_table_state: TableState::default().with_selected(None),
            scheduled_snapshot_table_state: TableState::default().with_selected(None),
            selected_group,
            focus: SnapshotFocus::ManualSnapshot,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let mut manual_snapshots_rows = Vec::new();
        let mut scheduled_snapshots_rows = Vec::new();

        // put these logic in a block to make sure the reference `group` drops early
        // otherwise, it will conflict with the `&mutable self` function call below
        {
            let Some(group) = get_sel_group(&self.btrfs_mgr, &self.selected_group) else {
                // TODO: no groups
                todo!();
            };
            let snapshots = group.get_snapshots();
            for x in snapshots {
                if x.get_type() == SnapshotType::Manually {
                    manual_snapshots_rows.push(Row::new([x.get_date(), x.get_time()]));
                } else {
                    scheduled_snapshots_rows.push(Row::new([
                        x.get_date(),
                        x.get_time(),
                        x.get_type().to_string(),
                    ]));
                }
            }
        }

        // determine the height of each block dynamically
        let manual_percentage = ((manual_snapshots_rows.len() * 100
            / (manual_snapshots_rows.len() + scheduled_snapshots_rows.len()))
            as u16)
            .clamp(20, 80);
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
            manual_snapshots_rows,
            focused && self.focus == SnapshotFocus::ManualSnapshot,
        );
        self.render_scheduled_block(
            frame,
            vertical_layout[1],
            scheduled_snapshots_rows,
            focused && self.focus == SnapshotFocus::ScheduledSnapshot,
        );
    }

    #[inline]
    /// returns (main_color, highlight_bg_color)
    fn get_color(focused: bool) -> (Color, Color) {
        if focused {
            (globals::FOCUSED_COLOR, globals::FOCUSED_HIGHLIHGT_BG_COLOR)
        } else {
            (globals::BODY_COLOR, globals::BODY_HIGHLIHGT_BG_COLOR)
        }
    }

    fn render_manual_block(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        rows: Vec<Row>,
        focused: bool,
    ) {
        let (main_color, highlight_bg_color) = Self::get_color(focused);
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
            let header =
                Row::new(["Date", "Time"]).style(Style::new().bold().italic().underlined());
            let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];
            // TODO: add highlight style and color
            let table = Table::new(rows, widths)
                .header(header)
                .column_spacing(1)
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
        let (main_color, highlight_bg_color) = Self::get_color(focused);
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
            let header =
                Row::new(["Date", "Time", "Type"]).style(Style::new().bold().italic().underlined());
            let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];
            // TODO: add highlight style and color
            let table = Table::new(rows, widths)
                .header(header)
                .column_spacing(1)
                .style(main_color);
            frame.render_stateful_widget(
                table.block(scheduled_block),
                area,
                &mut self.manual_snapshot_table_state,
            );
        };
    }
}
