use std::time::SystemTime;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

use crate::file_filter::FileFilter;

#[derive(Debug, PartialEq)]
struct FileState {
    pub modified: u64,
    pub size: u64,
}

impl FileState {
    pub fn from_metadata(metadata: &fs::Metadata) -> Self {
        if metadata.is_dir() {
            panic!("Directories are not supported!");
        }
        let size = metadata.len();
        let modified = metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        FileState { size, modified }
    }
}

#[derive(Debug, PartialEq)]
pub struct DirState {
    file_filter: FileFilter,
    path: PathBuf,
    files: HashMap<String, FileState>,
    subdirs: HashMap<String, DirState>,
}

impl DirState {
    pub fn from_dir(path: &PathBuf, file_filter: &FileFilter) -> Self {
        let mut files = HashMap::new();
        let mut subdirs = HashMap::new();
        for result in fs::read_dir(path).unwrap() {
            let entry = result.unwrap();
            if file_filter.is_file_excluded(&entry) {
                continue;
            }
            let filename = String::from(entry.file_name().to_string_lossy());
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                let subdir = path.join(&filename);
                subdirs.insert(filename, DirState::from_dir(&subdir, &file_filter));
            } else {
                files.insert(filename, FileState::from_metadata(&metadata));
            }
        }
        DirState { path: path.clone(), file_filter: file_filter.clone(), files, subdirs }
    }

    pub fn is_empty(&self) -> bool {
        self.files.len() == 0 && self.subdirs.len() == 0
    }

    pub fn are_contents_equal_to(&self, other: &DirState) -> bool {
        if self.files != other.files {
            return false;
        }
        if self.subdirs.len() != other.subdirs.len() {
            return false;
        }
        for (dirname, state) in self.subdirs.iter() {
            match other.subdirs.get(dirname) {
                None => { return false; },
                Some(other_state) => {
                    if !state.are_contents_equal_to(other_state) {
                        return false;
                    }
                }
            }
        }
        return true;
    }

    pub fn are_any_contents_newer_than(&self, other: &DirState) -> bool {
        for (filename, state) in self.files.iter() {
            if let Some(other_state) = other.files.get(filename) {
                if state.modified > other_state.modified {
                    return true;
                }
            }
        }
        for (dirname, state) in self.subdirs.iter() {
            if let Some(other_state) = other.subdirs.get(dirname) {
                if state.are_any_contents_newer_than(other_state) {
                    return true;
                }
            }
        }
        false
    }

    pub fn are_any_contents_older_than(&self, other: &DirState) -> bool {
        for (filename, state) in self.files.iter() {
            if let Some(other_state) = other.files.get(filename) {
                if state.modified < other_state.modified {
                    return true;
                }
            }
        }
        for (dirname, state) in self.subdirs.iter() {
            if let Some(other_state) = other.subdirs.get(dirname) {
                if state.are_any_contents_older_than(other_state) {
                    return true;
                }
            }
        }
        false
    }

    pub fn are_contents_generally_newer_than(&self, other: &DirState) -> bool {
        !self.is_empty() &&
        !self.are_any_contents_older_than(other) &&
        self.are_any_contents_newer_than(other)
    }

    pub fn copy_into(&self, dest: &PathBuf) {
        fs::create_dir_all(&dest).unwrap();
        for filename in self.files.keys() {
            let src_path = &self.path.join(filename);
            let dest_path = dest.join(filename);
            fs::copy(&src_path, &dest_path).unwrap();
        }
        for (dirname, dir) in self.subdirs.iter() {
            let dest_dir = dest.join(dirname);
            dir.copy_into(&dest_dir);
        }
    }

    pub fn remove_extraneous_files_from(&self, root: &PathBuf) {
        for result in fs::read_dir(root).unwrap() {
            let entry = result.unwrap();
            if self.file_filter.is_file_excluded(&entry) {
                continue;
            }
            let filepath = entry.path();
            let filename = String::from(entry.file_name().to_string_lossy());
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                if let Some(subdir) = self.subdirs.get(&filename) {
                    subdir.remove_extraneous_files_from(&filepath);
                } else {
                    fs::remove_dir_all(&filepath).unwrap();
                }
            } else {
                if !self.files.contains_key(&filename) {
                    fs::remove_file(&filepath).unwrap();
                }
            }
        }
    }
}

#[test]
fn test_dirstate() {
    let file_filter = FileFilter::default();

    // Setup: create/clear the temporary test dir.
    let tmp_dir = PathBuf::from(".test_dirstate");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).unwrap();
    }
    fs::create_dir(&tmp_dir).unwrap();

    // Copy the source test dir into the temp test dir.
    let src_dir = PathBuf::from("test-data/dirstate_test");
    let src_state = DirState::from_dir(&src_dir, &file_filter);
    assert!(!src_state.is_empty());
    src_state.copy_into(&tmp_dir);

    let mut tmp_state = DirState::from_dir(&tmp_dir, &file_filter);
    assert!(src_state.are_contents_equal_to(&tmp_state));
    assert!(tmp_state.are_contents_equal_to(&src_state));

    // Make a subdirectory in the temp test dir.
    let tmp_subdir = tmp_dir.join("another_subdir");
    fs::create_dir(&tmp_subdir).unwrap();
    assert!(DirState::from_dir(&tmp_subdir, &file_filter).is_empty());

    tmp_state = DirState::from_dir(&tmp_dir, &file_filter);
    assert!(!src_state.are_contents_equal_to(&tmp_state));

    // Add a file to the temp test dir.
    let tmp_file = tmp_dir.join("somefile");
    fs::write(&tmp_file, "blarg").unwrap();

    // Remove files from the temp test dir not in the source test dir.
    src_state.remove_extraneous_files_from(&tmp_dir);
    tmp_state = DirState::from_dir(&tmp_dir, &file_filter);
    assert!(src_state.are_contents_equal_to(&tmp_state));

    // Teardown: remove the temporary test dir.
    fs::remove_dir_all(&tmp_dir).unwrap();
}
