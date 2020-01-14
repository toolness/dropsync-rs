use std::time::SystemTime;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

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
    path: PathBuf,
    files: HashMap<String, FileState>,
    subdirs: HashMap<String, DirState>,
}

impl DirState {
    pub fn from_dir(path: &PathBuf) -> Self {
        let mut files = HashMap::new();
        let mut subdirs = HashMap::new();
        for result in fs::read_dir(path).unwrap() {
            let entry = result.unwrap();
            let filename = String::from(entry.file_name().to_string_lossy());
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                let subdir = path.join(&filename);
                subdirs.insert(filename, DirState::from_dir(&subdir));
            } else {
                files.insert(filename, FileState::from_metadata(&metadata));
            }
        }
        DirState { path: path.clone(), files, subdirs }
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

    pub fn are_contents_at_least_as_recent_as(&self, other: &DirState) -> bool {
        for (filename, state) in self.files.iter() {
            if let Some(other_state) = other.files.get(filename) {
                if state.modified < other_state.modified {
                    return false;
                }
            }
        }
        for (dirname, state) in self.subdirs.iter() {
            if let Some(other_state) = other.subdirs.get(dirname) {
                if !state.are_contents_at_least_as_recent_as(other_state) {
                    return false;
                }
            }
        }
        true
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
}
