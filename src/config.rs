use std::collections::HashMap;
use std::path::PathBuf;
use toml::Value;

use self::super::util;

pub type Config = HashMap<String, AppConfig>;

#[derive(Debug, PartialEq)]
pub struct AppConfig {
    pub name: String,
    pub path: PathBuf,
    pub dropbox_path: PathBuf,
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
            result.insert(name.clone(), AppConfig { name: name.clone(), path, dropbox_path });
        }
    } else {
        panic!("The top-level value of a config file should be a table!");
    }
    result
}

#[test]
fn test_load_config() {
    use std::fs;

    let toml_str = fs::read_to_string("test-data/sample_config.toml").expect("example config file should exist!");
    let configs = load_config("my_first_computer", &toml_str, &PathBuf::from("."));

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
