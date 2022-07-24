use std::path::Path;
use glob::Pattern;

#[derive(Debug, Clone, PartialEq)]
pub struct FileFilter {
    include_only: Option<Pattern>
}

impl FileFilter {
    pub fn is_file_included<T: AsRef<Path>>(&self, path: T) -> bool {
        if let Some(pattern) = &self.include_only {
            pattern.matches_path(path.as_ref())
        } else {
            true
        }
    }

    pub fn is_file_excluded<T: AsRef<Path>>(&self, path: T) -> bool {
        !self.is_file_included(path)
    }
}

impl Default for FileFilter {
    fn default() -> Self {
        Self { include_only: None }
    }
}
