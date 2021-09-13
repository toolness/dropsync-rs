use std::collections::HashMap;
use std::path::PathBuf;
use toml::Value;

use self::super::util;

#[derive(Debug, PartialEq)]
pub struct AppConfig {
    pub name: String,
    pub path: PathBuf,
    pub play_watch_dir: Option<PathBuf>,
    pub play_path: Option<PathBuf>,
    pub dropbox_path: PathBuf,
    pub disabled: bool,
}

impl AppConfig {
    pub fn validate(&self) {
        util::ensure_path_exists(&self.path);
        util::ensure_path_exists(&self.dropbox_path);
    }
}

fn normalize_path_slashes(path: &str) -> String {
    return path.replace("/", &std::path::MAIN_SEPARATOR.to_string());
}

fn get_app_config_bool(config: &Value, hostname: &str, key: &str, default: bool) -> bool {
    let host_config = config.get(hostname);
    if let Some(Value::Table(table)) = host_config {
        if let Some(Value::Boolean(s)) = table.get(key) {
            return *s;
        }
    }
    if let Some(Value::Boolean(s)) = config.get(key) {
        return *s;
    }
    default
}

fn get_optional_app_config_str<'a>(config: &'a Value, hostname: &str, key: &str) -> Option<&'a str> {
    let host_config = config.get(hostname);
    if let Some(Value::Table(table)) = host_config {
        if let Some(Value::String(s)) = table.get(key) {
            return Some(s);
        }
    }
    if let Some(Value::String(s)) = config.get(key) {
        return Some(s);
    }
    None
}

fn get_app_config_str<'a>(config: &'a Value, app_name: &str, hostname: &str, key: &str) -> &'a str {
    if let Some(s) = get_optional_app_config_str(config, hostname, key) {
        s
    } else {
        let err = format!("Unable to find config key '{}' for app '{}' and hostname '{}'!", key, app_name, hostname);
        panic!("{}", err);
    }
}

pub fn load_config(hostname: &str, config_toml: &str, root_dropbox_path: &PathBuf) -> HashMap<String, AppConfig> {
    let config = config_toml.parse::<Value>().unwrap();
    let mut result = HashMap::new();
    if let Value::Table(table) = config {
        for entry in table.iter() {
            let (name, app_config) = entry;
            let path = PathBuf::from(get_app_config_str(app_config, name, hostname, "path"));
            let norm_dropbox_path = normalize_path_slashes(get_app_config_str(app_config, name, hostname, "dropbox_path"));
            let rel_dropbox_path = PathBuf::from(norm_dropbox_path);
            let dropbox_path = root_dropbox_path.join(rel_dropbox_path);
            let disabled = get_app_config_bool(app_config, hostname, "disabled", false);
            let play_root_path = if let Some(play_root_path_str) = get_optional_app_config_str(app_config, hostname, "play_root_path") {
                Some(PathBuf::from(play_root_path_str))
            } else {
                None
            };
            let play_path = if let Some(play_path_str) = get_optional_app_config_str(app_config, hostname, "play_path") {
                Some(maybe_join_paths(&play_root_path, normalize_path_slashes(play_path_str).into()))
            } else {
                None
            };
            result.insert(name.clone(), AppConfig {
                name: name.clone(),
                path,
                dropbox_path,
                disabled,
                play_path,
                play_watch_dir: play_root_path
            });
        }
    } else {
        panic!("The top-level value of a config file should be a table!");
    }
    result
}

fn maybe_join_paths(first: &Option<PathBuf>, second: PathBuf) -> PathBuf {
    if let Some(root_path) = first {
        root_path.join(second)
    } else {
        second
    }
}

#[test]
fn test_load_config() {
    use std::fs;

    let toml_str = fs::read_to_string("test-data/sample_config.toml").expect("example config file should exist!");
    let configs = load_config("my_first_computer", &toml_str, &PathBuf::from("."));

    let mut expected = HashMap::new();
    expected.insert(
        String::from("app1"),
        AppConfig { name: String::from("app1"), path: PathBuf::from("C:\\myapp1\\stuff"), dropbox_path: PathBuf::from("./MyAppData/app1"), disabled: false, play_path: None, play_watch_dir: None }
    );
    expected.insert(
        String::from("app2"),
        AppConfig { name: String::from("app2"), path: PathBuf::from("F:\\myapp2\\stuff"), dropbox_path: PathBuf::from("./MyAppData/app2"), disabled: false, play_path: None, play_watch_dir: None }
    );
    expected.insert(
        String::from("app3"),
        AppConfig { name: String::from("app3"), path: PathBuf::from("G:\\app3"), dropbox_path: PathBuf::from("./MyAppData/app3"), disabled: true, play_path: None, play_watch_dir: None }
    );

    assert_eq!(expected, configs);
}
