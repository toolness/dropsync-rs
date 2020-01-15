use std::path::PathBuf;
use std::process::Command;

pub fn open_in_explorer(path: &PathBuf) {
    if cfg!(target_os = "windows") {
        println!("Opening {}.", path.to_string_lossy());
        Command::new("explorer")
          .arg(&path.as_os_str())
          .spawn()
          .unwrap();
    } else {
        unimplemented!();
    }
}
