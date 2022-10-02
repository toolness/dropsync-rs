use std::path::PathBuf;
use std::process::Command;

pub fn open_in_explorer(path: &PathBuf) {
    println!("Opening {}.", path.to_string_lossy());
    if cfg!(target_os = "windows") {
        Command::new("explorer")
          .arg(&path.as_os_str())
          .spawn()
          .unwrap();
    } else if cfg!(target_os = "macos") {
        Command::new("open")
          .arg(&path.as_os_str())
          .spawn()
          .unwrap();
    } else {
        println!("Oops, I don't know how to open it on this OS.");
        println!("Please open it yourself. Sorry!");
    }
}
