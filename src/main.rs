use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::fs;
use serde::Deserialize;
use sysinfo::{System, SystemExt, ProcessExt};

mod dir_state;
mod dropbox;
mod util;
mod ask;
mod config;
mod explorer;

use dir_state::DirState;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const USAGE: &'static str = "
Synchronize app files with Dropbox.

Usage:
  dropsync
  dropsync config
  dropsync explore <app>
  dropsync play <app>
  dropsync --version
  dropsync (-h | --help)

Options:
  -h --help     Show this screen.
  --version     Show version.
";

const WATCH_DIR_MAX_SECONDS: u64 = 3;

#[derive(Debug, Deserialize)]
struct Args {
    cmd_explore: bool,
    cmd_config: bool,
    cmd_play: bool,
    arg_app: Option<String>,
}

#[derive(PartialEq)]
enum SyncResult {
    AlreadySynced,
    AppNewerThanDropbox,
    DropboxNewerThanApp,
    BothEmpty,
    Conflict,
}

#[derive(PartialEq, Copy, Clone)]
enum ConflictChoice {
    UseApp,
    UseDropbox,
    Explore
}

static CONFLICT_CHOICES: [ask::Choice<ConflictChoice>; 3] = [
    ask::Choice { name: "use app", value: ConflictChoice::UseApp },
    ask::Choice { name: "use dropbox", value: ConflictChoice::UseDropbox },
    ask::Choice { name: "explore", value: ConflictChoice::Explore },
];

fn sync_app(app: &config::AppConfig, confirm_if_app_is_newer: bool) -> SyncResult {
    let dir_state = DirState::from_dir(&app.path);
    let dropbox_dir_state = DirState::from_dir(&app.dropbox_path);
    if dir_state.are_contents_equal_to(&dropbox_dir_state) {
        println!("  App state matches Dropbox. Nothing to do!");
        SyncResult::AlreadySynced
    } else {
        if dir_state.are_contents_generally_newer_than(&dropbox_dir_state) {
            println!("  App state is newer than Dropbox.");
            copy_files_with_maybe_confirmation(&dir_state, &app.dropbox_path, confirm_if_app_is_newer);
            SyncResult::AppNewerThanDropbox
        } else if dropbox_dir_state.are_contents_generally_newer_than(&dir_state) {
            println!("  Dropbox state is newer than app.");
            copy_files_with_maybe_confirmation(&dropbox_dir_state, &app.path, true);
            SyncResult::DropboxNewerThanApp
        } else if dir_state.is_empty() && dropbox_dir_state.is_empty() {
            println!("  Both Dropbox and app state are empty. Nothing to do!");
            SyncResult::BothEmpty
        } else {
            println!("  App and Dropbox state are in conflict; manual resolution required.");
            let choice = ask::ask_with_choices("  ", "How do you want to proceed? ", &CONFLICT_CHOICES);
            match choice {
                ConflictChoice::UseApp => {
                    copy_files_with_maybe_confirmation(&dir_state, &app.dropbox_path, false);
                    SyncResult::AppNewerThanDropbox
                }
                ConflictChoice::UseDropbox => {
                    copy_files_with_maybe_confirmation(&dropbox_dir_state, &app.path, false);
                    SyncResult::DropboxNewerThanApp
                }
                ConflictChoice::Explore => {
                    explorer::open_in_explorer(&app.path);
                    explorer::open_in_explorer(&app.dropbox_path);
                    SyncResult::Conflict
                }
            }
        }
    }
}

fn copy_files_with_maybe_confirmation(from_dir: &DirState, to_dir: &PathBuf, should_ask: bool) {
    let yes = if should_ask {
        ask::ask_yes_or_no("  Proceed with synchronization (y/n) ? ")
    } else {
        println!("  Synchronizing files.");
        true
    };
    if yes {
        from_dir.copy_into(to_dir);
        from_dir.remove_extraneous_files_from(to_dir);
    } else {
        println!("  Okay, not doing anything.");
    }
}

fn play(executable: &PathBuf, watch_dir: &Option<PathBuf>) {
    let executable_dir = executable.parent()
        .expect("executable should have a parent directory")
        .to_path_buf();
    let mut child = Command::new(executable)
        .current_dir(&executable_dir)
        .stdin(Stdio::inherit())
        .spawn()
        .expect("process failed to execute");
    child.wait().expect("failed to wait on child");

    let final_watch_dir = watch_dir.as_ref().unwrap_or(&executable_dir);
    let mut sys = System::new();
    let mut seconds_without_exe = 0;
    println!("Waiting for no processes to be running in app directory for {} seconds.", WATCH_DIR_MAX_SECONDS);
    while seconds_without_exe < WATCH_DIR_MAX_SECONDS {
        std::thread::sleep(std::time::Duration::from_secs(1));
        seconds_without_exe += 1;
        sys.refresh_processes();
        for (_, process) in sys.processes() {
            if process.exe().starts_with(final_watch_dir) {
                seconds_without_exe = 0;
            }
        }
    }
    println!("Looks like the app is finished.");
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

    if args.cmd_explore || args.cmd_play {
        let app_name = args.arg_app.unwrap();
        if let Some(config) = util::get_case_insensitive(&app_configs, &app_name) {
            if args.cmd_explore {
                explorer::open_in_explorer(&config.path);
                explorer::open_in_explorer(&config.dropbox_path);
            } else {
                assert_eq!(args.cmd_play, true);
                if let Some(play_path) = &config.play_path {
                    config.validate();
                    if sync_app(config, true) == SyncResult::Conflict {
                        rprompt::prompt_reply_stdout("Press enter once you've resolved the conflict.").unwrap();
                    }
                    play(play_path, &config.play_watch_dir);
                    // Don't ask anything if the app is newer, since we fully expect that to be the case.
                    sync_app(config, false);
                } else {
                    println!("No play_path is defined for {}!", app_name);
                    std::process::exit(1);
                }
            }
        } else {
            let app_names = app_configs.keys().map(|s| format!("'{}'", s)).collect::<Vec<String>>();
            println!("App '{}' not found! Please choose from {}.", &app_name, app_names.join(", "));
            std::process::exit(1);
        }
    } else {
        let mut sorted_configs: Vec<&config::AppConfig> = app_configs.values().collect();
        sorted_configs.sort_by(|a, b| a.name.cmp(&b.name));

        for config in sorted_configs.iter().filter(|cfg| !cfg.disabled) {
            println!("Syncing app {}.", config.name);
            config.validate();
            sync_app(config, true);
        }
    }
}
