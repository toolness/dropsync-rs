use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use std::fs;
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

#[derive(Debug)]
struct FileInfo {
    modified: u64,
    size: u64,
}

impl FileInfo {
    fn from_dir_entry(entry: &fs::DirEntry) -> Self {
        let metadata = entry.metadata().unwrap();
        if metadata.is_dir() {
            panic!("Directories are not supported!");
        }
        let size = metadata.len();
        let modified = metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        FileInfo { size, modified }
    }
}

#[derive(Debug)]
struct DirState {
    files: HashMap<String, FileInfo>,
}

impl DirState {
    fn from_dir(path: &PathBuf) -> Self {
        let mut files = HashMap::new();
        for result in fs::read_dir(path).unwrap() {
            let entry = result.unwrap();
            let filename = String::from(entry.file_name().to_string_lossy());
            files.insert(filename, FileInfo::from_dir_entry(&entry));
        }
        DirState { files }
    }
}

impl AppConfig {
    pub fn validate(&self) {
        ensure_path_exists(&self.path);
        ensure_path_exists(&self.dropbox_path);
    }

    pub fn sync(&self) {
        let dir_state = DirState::from_dir(&self.path);
        let dropbox_dir_state = DirState::from_dir(&self.dropbox_path);
        println!("our dir state: {:?}", dir_state);
        println!("dropbox dir state: {:?}", dropbox_dir_state);
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

    let toml_str = fs::read_to_string(cfg_file).unwrap();
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
