use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use toml::Value;

mod dir_state;
mod dropbox;
mod util;

use dir_state::DirState;

#[derive(Debug, PartialEq)]
struct AppConfig {
    pub name: String,
    pub path: PathBuf,
    pub dropbox_path: PathBuf,
}

impl AppConfig {
    pub fn validate(&self) {
        util::ensure_path_exists(&self.path);
        util::ensure_path_exists(&self.dropbox_path);
    }

    pub fn sync(&self) {
        let dir_state = DirState::from_dir(&self.path);
        let dropbox_dir_state = DirState::from_dir(&self.dropbox_path);
        if dir_state.are_contents_equal_to(&dropbox_dir_state) {
            println!("  App state matches Dropbox. Nothing to do!");
        } else {
            if dir_state.are_contents_generally_newer_than(&dropbox_dir_state) {
                println!("  App state is newer than Dropbox.");
                copy_files_with_confirmation(&dir_state, &self.dropbox_path);
            } else if dropbox_dir_state.are_contents_generally_newer_than(&dir_state) {
                println!("  Dropbox state is newer than app.");
                copy_files_with_confirmation(&dropbox_dir_state, &self.path);
            } else if dir_state.is_empty() && dropbox_dir_state.is_empty() {
                println!("  Both Dropbox and app state are empty. Nothing to do!");
            } else {
                println!("  App and Dropbox state are in conflict; manual resolution required.");
                if ask_yes_or_no("  Open folders in explorer (y/n) ? ") {
                    open_in_explorer(&self.path);
                    open_in_explorer(&self.dropbox_path);
                }
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

fn parse_yes_or_no(value: &str) -> Option<bool> {
    let lower = value.to_lowercase();
    if lower.starts_with("y") {
        Some(true)
    } else if lower.starts_with("n") {
        Some(false)
    } else {
        None
    }
}

fn ask_yes_or_no(prompt: &str) -> bool {
    loop {
        let reply = rprompt::prompt_reply_stdout(prompt).unwrap();
        if let Some(response) = parse_yes_or_no(&reply) {
            return response;
        }
    }
}

fn copy_files_with_confirmation(from_dir: &DirState, to_dir: &PathBuf) {
    let yes = ask_yes_or_no("  Proceed with synchronization (y/n) ? ");
    if yes {
        from_dir.copy_into(to_dir);
    } else {
        println!("  Okay, not doing anything.");
    }
}

fn get_app_config_str<'a>(config: &'a Value, app_name: &str, hostname: &str, key: &str) -> &'a str {
    let host_config = config.get(hostname);
    if let Some(Value::Table(table)) = host_config {
        if let Some(Value::String(s)) = table.get(key) {
            return s;
        }
    }
    if let Some(Value::String(s)) = config.get(key) {
        return s;
    }
    let err = format!("Unable to find config key '{}' for app '{}' and hostname '{}'!", key, app_name, hostname);
    panic!(err);
}

fn normalize_path_slashes(path: &str) -> String {
    return path.replace("/", &std::path::MAIN_SEPARATOR.to_string());
}

fn load_config(hostname: &str, config: Value, root_dropbox_path: &PathBuf) -> HashMap<String, AppConfig> {
    let mut result = HashMap::new();
    if let Value::Table(table) = config {
        for entry in table.iter() {
            let (name, app_config) = entry;
            let path = PathBuf::from(get_app_config_str(app_config, name, hostname, "path"));
            let norm_dropbox_path = normalize_path_slashes(get_app_config_str(app_config, name, hostname, "dropbox_path"));
            let rel_dropbox_path = PathBuf::from(norm_dropbox_path);
            let dropbox_path = root_dropbox_path.join(rel_dropbox_path);
            result.insert(name.clone(), AppConfig { name: name.clone(), path, dropbox_path });
        }
    } else {
        panic!("The top-level value of a config file should be a table!");
    }
    result
}

fn load_config_from_dropbox_dir(hostname: &str, root_dropbox_path: &PathBuf) -> HashMap<String, AppConfig> {
    let cfg_file = root_dropbox_path.join("dropsync.toml");
    util::ensure_path_exists(&cfg_file);

    println!("Loading configuration from {}.", cfg_file.to_string_lossy());

    let toml_str = fs::read_to_string(cfg_file).unwrap();
    let value = toml_str.parse::<Value>().unwrap();

    load_config(hostname, value, root_dropbox_path)
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
        config.sync();
    }
}

#[test]
fn test_load_config() {
    let toml_str = fs::read_to_string("test-data/sample_config.toml").expect("example config file should exist!");
    let value = toml_str.parse::<Value>().unwrap();
    let configs = load_config("my_first_computer", value, &PathBuf::from("."));

    let mut expected = HashMap::new();
    expected.insert(
        String::from("app1"),
        AppConfig { name: String::from("app1"), path: PathBuf::from("C:\\myapp1\\stuff"), dropbox_path: PathBuf::from("./MyAppData/app1") }
    );
    expected.insert(
        String::from("app2"),
        AppConfig { name: String::from("app2"), path: PathBuf::from("F:\\myapp2\\stuff"), dropbox_path: PathBuf::from("./MyAppData/app2") }
    );

    assert_eq!(expected, configs);
}
