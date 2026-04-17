use std::path::PathBuf;

pub struct Subvolume {
    path: PathBuf,
}

impl Subvolume {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
