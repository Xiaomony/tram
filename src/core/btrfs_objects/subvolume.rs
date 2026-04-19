use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Subvolume {
    path: PathBuf,
}

impl Subvolume {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn get_path(&self) -> &Path {
        self.path.as_ref()
    }
}

impl<T: AsRef<Path>> PartialEq<T> for Subvolume {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.path == other.as_ref()
    }
}
