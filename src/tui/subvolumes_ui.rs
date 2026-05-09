use std::{cell::RefCell, rc::Rc};

use ratatui::{Frame, layout::Rect};

use crate::core::btrfs_manager::BtrfsManager;

pub struct SubvolumesUI {
    btrfs_mgr: Rc<RefCell<BtrfsManager>>,
}

impl SubvolumesUI {
    pub fn new(btrfs_mgr: Rc<RefCell<BtrfsManager>>) -> Self {
        Self { btrfs_mgr }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {}
}
