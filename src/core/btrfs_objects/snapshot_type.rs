use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SnapshotType {
    Manually,
    Daily,
    Monthly,
    Weekly,
    Boot,
}

impl SnapshotType {
    pub fn get_type(string: &str) -> Option<Self> {
        match string {
            "Manually" | "manually" => Some(SnapshotType::Manually),
            "Daily" | "daily" => Some(SnapshotType::Daily),
            "Monthly" | "monthly" => Some(SnapshotType::Monthly),
            "Weekly" | "weekly" => Some(SnapshotType::Weekly),
            "Boot" | "boot" => Some(SnapshotType::Boot),
            _ => None,
        }
    }
}

impl Display for SnapshotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for SnapshotType {
    fn as_ref(&self) -> &str {
        use SnapshotType::*;
        match self {
            Manually => "manually",
            Daily => "daily",
            Weekly => "weekly",
            Monthly => "monthly",
            Boot => "boot",
        }
    }
}
