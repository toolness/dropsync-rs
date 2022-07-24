use std::fs::DirEntry;
use glob::Pattern;

#[derive(Debug, Clone, PartialEq)]
pub struct FileFilter {
    include_only: Option<Pattern>
}

impl FileFilter {
    pub fn is_file_included(&self, entry: &DirEntry) -> bool {
        if let Some(pattern) = &self.include_only {
            pattern.matches_path(&entry.path())
        } else {
            true
        }
    }

    pub fn is_file_excluded(&self, entry: &DirEntry) -> bool {
        !self.is_file_included(entry)
    }
}

impl Default for FileFilter {
    fn default() -> Self {
        Self { include_only: None }
    }
}
