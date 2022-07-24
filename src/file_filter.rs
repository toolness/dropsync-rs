use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct FileFilter();

impl FileFilter {
    pub fn is_file_included(&self, _path: &PathBuf) -> bool {
        true
    }

    pub fn is_file_excluded(&self, path: &PathBuf) -> bool {
        !self.is_file_included(&path)
    }
}

impl Default for FileFilter {
    fn default() -> Self {
        Self()
    }
}
