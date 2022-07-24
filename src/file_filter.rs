use std::path::PathBuf;

pub trait FileFilter {
    fn is_file_included(&self, path: &PathBuf) -> bool;

    fn is_file_excluded(&self, path: &PathBuf) -> bool {
        !self.is_file_included(&path)
    }
}

pub struct DefaultFileFilter();

impl FileFilter for DefaultFileFilter {
    fn is_file_included(&self, _path: &PathBuf) -> bool {
        true
    }
}
