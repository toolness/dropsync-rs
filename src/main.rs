use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::read_to_string;
use toml::Value;

#[derive(Debug, PartialEq)]
struct AppConfig {
    pub name: String,
    pub path: PathBuf,
    pub dropbox_path: PathBuf,
}

fn ensure_path_exists(value: &PathBuf) {
    if !value.exists() {
        let err = format!("Path '{}' does not exist!", value.to_string_lossy());
        panic!(err);
    }
}

fn get_dropbox_dir() -> PathBuf {
    let mut home_dir = dirs::home_dir().expect("User should have a home directory!");
    home_dir.push("Dropbox");
    ensure_path_exists(&home_dir);
    home_dir
}

impl AppConfig {
    pub fn validate(&self) {
        ensure_path_exists(&self.path);
        ensure_path_exists(&self.dropbox_path);
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

fn load_config(hostname: &str, config: Value, root_dropbox_path: &PathBuf) -> HashMap<String, AppConfig> {
    let mut result = HashMap::new();
    if let Value::Table(table) = config {
        for entry in table.iter() {
            let (name, app_config) = entry;
            let path = PathBuf::from(get_app_config_str(app_config, name, hostname, "path"));
            let rel_dropbox_path = PathBuf::from(get_app_config_str(app_config, name, hostname, "dropbox_path"));
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
    ensure_path_exists(&cfg_file);

    println!("Loading configuration from {}.", cfg_file.to_string_lossy());

    let toml_str = read_to_string(cfg_file).unwrap();
    let value = toml_str.parse::<Value>().unwrap();

    load_config(hostname, value, root_dropbox_path)
}

fn main() {
    let raw_hostname = gethostname::gethostname();
    let hostname = raw_hostname.to_string_lossy();

    println!("Syncing apps on host {}.", hostname);

    let dropbox_dir = get_dropbox_dir();
    let app_configs = load_config_from_dropbox_dir(&hostname, &dropbox_dir);

    for config in app_configs.values() {
        println!("Syncing app {}.", config.name);
        config.validate();
    }
}

#[test]
fn test_load_config() {
    let toml_str = read_to_string("test-data/sample_config.toml").expect("example config file should exist!");
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
