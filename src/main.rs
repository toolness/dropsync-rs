use std::path::PathBuf;
use std::fs;
use std::process::Command;

mod dir_state;
mod dropbox;
mod util;
mod ask;
mod config;

use dir_state::DirState;

fn sync_app(app: &config::AppConfig) {
    let dir_state = DirState::from_dir(&app.path);
    let dropbox_dir_state = DirState::from_dir(&app.dropbox_path);
    if dir_state.are_contents_equal_to(&dropbox_dir_state) {
        println!("  App state matches Dropbox. Nothing to do!");
    } else {
        if dir_state.are_contents_generally_newer_than(&dropbox_dir_state) {
            println!("  App state is newer than Dropbox.");
            copy_files_with_confirmation(&dir_state, &app.dropbox_path);
        } else if dropbox_dir_state.are_contents_generally_newer_than(&dir_state) {
            println!("  Dropbox state is newer than app.");
            copy_files_with_confirmation(&dropbox_dir_state, &app.path);
        } else if dir_state.is_empty() && dropbox_dir_state.is_empty() {
            println!("  Both Dropbox and app state are empty. Nothing to do!");
        } else {
            println!("  App and Dropbox state are in conflict; manual resolution required.");
            if ask::ask_yes_or_no("  Open folders in explorer (y/n) ? ") {
                open_in_explorer(&app.path);
                open_in_explorer(&app.dropbox_path);
            }
        }
    }
}

fn open_in_explorer(path: &PathBuf) {
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

fn copy_files_with_confirmation(from_dir: &DirState, to_dir: &PathBuf) {
    let yes = ask::ask_yes_or_no("  Proceed with synchronization (y/n) ? ");
    if yes {
        from_dir.copy_into(to_dir);
    } else {
        println!("  Okay, not doing anything.");
    }
}

fn load_config_from_dropbox_dir(hostname: &str, root_dropbox_path: &PathBuf) -> config::Config {
    let cfg_file = root_dropbox_path.join("dropsync.toml");
    util::ensure_path_exists(&cfg_file);

    println!("Loading configuration from {}.", cfg_file.to_string_lossy());

    let toml_str = fs::read_to_string(cfg_file).unwrap();
    config::load_config(hostname, &toml_str, root_dropbox_path)
}

fn main() {
    let raw_hostname = gethostname::gethostname();
    let hostname = raw_hostname.to_string_lossy();

    println!("Syncing apps on host {}.", hostname);

    let dropbox_dir = dropbox::get_dropbox_dir();
    let app_configs = load_config_from_dropbox_dir(&hostname, &dropbox_dir);

    for config in app_configs.values() {
        println!("Syncing app {}.", config.name);
        config.validate();
        sync_app(config);
    }
}
