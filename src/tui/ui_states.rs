use ratatui::text::{Line, Text};
use std::fmt::Display;

#[derive(Clone, Copy)]
pub enum Focus {
    Menu,
    ManualSnapshots,
    ScheduledSnapshots,
}

#[derive(Clone, Copy)]
pub enum Menu {
    Snapshots,
    Groups,
    Subvolumes,
    BrokenSnapshots,
    Settings,
}

impl Display for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref(),)
    }
}
impl AsRef<str> for Menu {
    fn as_ref(&self) -> &'static str {
        use Menu::*;
        match self {
            Snapshots => "  Snapshots ",
            Groups => "  Groups ",
            Subvolumes => " 󰨖 Subvolumes ",
            BrokenSnapshots => " 󰜺 Broken Snapshots ",
            Settings => "  Settings ",
        }
    }
}
impl From<Menu> for &str {
    fn from(val: Menu) -> Self {
        use Menu::*;
        match val {
            Snapshots => "  Snapshots ",
            Groups => "  Groups ",
            Subvolumes => " 󰨖 Subvolumes ",
            BrokenSnapshots => " 󰜺 Broken Snapshots ",
            Settings => "  Settings ",
        }
    }
}

impl<'a> From<Menu> for Line<'a> {
    fn from(val: Menu) -> Self {
        Line::from(Into::<&str>::into(val))
    }
}

impl<'a> From<Menu> for Text<'a> {
    fn from(val: Menu) -> Self {
        Text::from(Into::<&str>::into(val))
    }
}
