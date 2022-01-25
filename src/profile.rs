use std::collections::HashMap;

use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;

use crate::parser::{prop_string, Prop};

static PROFILES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/profiles");

pub static PROFILES: Lazy<HashMap<String, Preset>> = Lazy::new(|| {
  let parse_title = prop_string("profile_title");
  let parse_notes = prop_string("profile_notes");

  let mut map = HashMap::default();
  for file in PROFILES_DIR.files() {
    let mut preset = Preset::default();
    let file_name = file.path().file_name().unwrap().to_str().unwrap();
    let data = file.contents_utf8().unwrap().to_string();

    // filter by "Advanced" profile (settings_2c)
    if data.find("settings_2c").is_none() {
      continue;
    }

    for line in data.lines() {
      if line.starts_with("advanced_shot") {
        let end = line.len() - 1;
        preset.data = format!("{}\n", line[15..end].to_string());
      } else if line.starts_with("profile_title") {
        if let Ok((_, Prop::Unknown((_, title)))) = parse_title(line.as_bytes()) {
          preset.title = title;
        }
      } else if line.starts_with("profile_notes") {
        if let Ok((_, Prop::Unknown((_, notes)))) = parse_notes(line.as_bytes()) {
          preset.notes = notes;
        }
      }
    }
    map.insert(file_name.to_string(), preset);
  }
  map
});

#[derive(Clone, Default)]
pub struct Preset {
  pub title: String,
  pub notes: String,
  pub data: String,
}
