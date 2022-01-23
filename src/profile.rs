use std::collections::HashMap;

use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;

static PROFILES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/profiles");

pub static PROFILES: Lazy<HashMap<String, Preset>> = Lazy::new(|| {
  let mut map = HashMap::default();
  for file in PROFILES_DIR.files() {
    let data = file.contents_utf8().unwrap().to_string();
    for line in data.lines() {
      if line.starts_with("advanced_shot") && !line.ends_with("{}") {
        let end = line.len() - 1;
        let name = file.path().file_name().unwrap().to_str().unwrap();
        map.insert(
          name.to_string(),
          Preset {
            name: name.into(),
            data: format!("{}\n", line[15..end].to_string()),
          },
        );
        break;
      }
    }
  }
  map
});

pub struct Preset {
  pub name: String,
  pub data: String,
}
