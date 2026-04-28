use std::{cell::RefCell, rc::Rc};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, HorizontalAlignment, Layout},
    style::{Color, Modifier},
    widgets::{Block, BorderType, List, ListState, Paragraph},
};

use crate::core::{btrfs_manager::BtrfsManager, error::AppResult};
use crate::globals;

pub struct AppTUI {
    btrfs_mgr: Rc<RefCell<BtrfsManager>>,
    menu_state: ListState,
}

impl AppTUI {
    pub fn new(btrfs_mgr: Rc<RefCell<BtrfsManager>>) -> Self {
        Self {
            btrfs_mgr,
            menu_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(79)])
            .spacing(1)
            .split(frame.area());

        // render menu list
        {
            let menu_block = Block::bordered()
                .title(" 󰍜 Menu ")
                .title_alignment(HorizontalAlignment::Center)
                .border_type(BorderType::Rounded);
            let list = List::new(globals::MENU_ITEMS)
                .style(Color::Blue)
                .highlight_style(Modifier::REVERSED)
                .highlight_spacing(ratatui::widgets::HighlightSpacing::WhenSelected)
                .block(menu_block);
            frame.render_stateful_widget(list, horizontal_layout[0], &mut self.menu_state);
        }

        {
            let body_block = Block::bordered()
                .title(globals::MENU_ITEMS[self.menu_state.selected().unwrap()])
                .title_alignment(HorizontalAlignment::Center)
                .border_type(BorderType::Rounded);
            let body = Paragraph::new("some text").block(body_block);
            frame.render_widget(body, horizontal_layout[1]);
        }
    }

    // returns whether the program should exit
    pub fn read_events(&mut self) -> AppResult<bool> {
        if let Event::Key(key_event) = event::read()? {
            use KeyCode::*;
            match key_event.code {
                Char('k') | Up => self.menu_state.select_previous(),
                Char('j') | Down => self.menu_state.select_next(),
                Char('q') | Esc => return Ok(true),
                _ => (),
            }
        }
        Ok(false)
    }
}
