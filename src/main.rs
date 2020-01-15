use std::path::PathBuf;
use std::fs;
use serde::Deserialize;

mod dir_state;
mod dropbox;
mod util;
mod ask;
mod config;
mod explorer;

use dir_state::DirState;

const VERSION: &'static str = "1.0.0";

const USAGE: &'static str = "
Synchronize app files with Dropbox.

Usage:
  dropsync
  dropsync config
  dropsync explore <app>
  dropsync --version
  dropsync (-h | --help)
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_explore: bool,
    cmd_config: bool,
    arg_app: Option<String>,
}

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
                explorer::open_in_explorer(&app.path);
                explorer::open_in_explorer(&app.dropbox_path);
            }
        }
    }
}

fn copy_files_with_confirmation(from_dir: &DirState, to_dir: &PathBuf) {
    let yes = ask::ask_yes_or_no("  Proceed with synchronization (y/n) ? ");
    if yes {
        from_dir.copy_into(to_dir);
        from_dir.remove_extraneous_files_from(to_dir);
    } else {
        println!("  Okay, not doing anything.");
    }
}

fn main() {
    let version = VERSION.to_owned();
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.version(Some(version)).deserialize())
        .unwrap_or_else(|e| e.exit());

    let raw_hostname = gethostname::gethostname();
    let hostname = raw_hostname.to_string_lossy();

    let dropbox_dir = dropbox::get_dropbox_dir();
    let cfg_file = dropbox_dir.join("dropsync.toml");
    util::ensure_path_exists(&cfg_file);

    if args.cmd_config {
        explorer::open_in_explorer(&cfg_file);
        return;
    }

    println!("Loading config for {} from {}.", hostname, cfg_file.to_string_lossy());

    let toml_str = fs::read_to_string(cfg_file).unwrap();
    let app_configs = config::load_config(&hostname, &toml_str, &dropbox_dir);

    if args.cmd_explore {
        let app_name = args.arg_app.unwrap();
        if let Some(config) = app_configs.get(&app_name) {
            explorer::open_in_explorer(&config.path);
            explorer::open_in_explorer(&config.dropbox_path);
        } else {
            let app_names = app_configs.keys().map(|s| format!("'{}'", s)).collect::<Vec<String>>();
            println!("App '{}' not found! Please choose from {}.", &app_name, app_names.join(", "));
            std::process::exit(1);
        }
    } else {
        for config in app_configs.values() {
            println!("Syncing app {}.", config.name);
            config.validate();
            sync_app(config);
        }
    }
}
