use std::path::PathBuf;

pub fn ensure_path_exists(value: &PathBuf) {
  if !value.exists() {
      let err = format!("Path '{}' does not exist!", value.to_string_lossy());
      panic!("{}", err);
  }
}
