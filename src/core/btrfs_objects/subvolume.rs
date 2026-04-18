use std::path::PathBuf;

#[derive(Debug)]
pub struct Subvolume {
    path: PathBuf,
}

impl Subvolume {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
