use std::path::PathBuf;

use self::super::util;

pub fn get_dropbox_dir() -> PathBuf {
    let mut home_dir = dirs::home_dir().expect("User should have a home directory!");
    home_dir.push("Dropbox");
    util::ensure_path_exists(&home_dir);
    home_dir
}
