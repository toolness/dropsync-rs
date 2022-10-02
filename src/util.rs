use std::path::PathBuf;
use std::collections::HashMap;

pub fn ensure_path_exists(value: &PathBuf) {
  if !value.exists() {
      let err = format!("Path '{}' does not exist!", value.to_string_lossy());
      panic!("{}", err);
  }
}

pub fn get_case_insensitive<'a, T>(map: &'a HashMap<String, T>, key: &String) -> Option<&'a T> {
  let key_lowercase = key.to_lowercase();

  for (k, v) in map {
      if k.to_lowercase() == key_lowercase {
          return Some(v);
      }
  }
  None
}

#[test]
fn test_get_case_insensitive() {
  let mut h = HashMap::new();

  h.insert("Boop".to_owned(), 1);

  assert_eq!(get_case_insensitive(&h, &"BOOP".to_owned()), Some(&1));
  assert_eq!(get_case_insensitive(&h, &"meh".to_owned()), None);
}

pub fn get_primary_hostname<T: AsRef<str>>(hostname: T) -> String {
  if let Some(index) = hostname.as_ref().find(".") {
    hostname.as_ref().split_at(index).0.into()
  } else {
    hostname.as_ref().into()
  }
}

#[test]
fn test_get_primary_hostname() {
  assert_eq!(get_primary_hostname("boop"), "boop".to_owned());
  assert_eq!(get_primary_hostname("boop.local"), "boop".to_owned());
  assert_eq!(get_primary_hostname("boop.blap.local"), "boop".to_owned());
}
