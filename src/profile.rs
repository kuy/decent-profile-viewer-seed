use std::collections::HashMap;

use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;

use crate::parser::{prop_string, Prop, PumpType, Step, TransitionType};

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

pub type PositionList = Vec<(f64, f64, f64, f64)>;

pub fn analyze(steps: &Vec<Step>) -> (PositionList, PositionList, PositionList, f64) {
  let mut temperature_pos: PositionList = vec![];
  let mut last_temperature_pos: Option<(f64, f64, f64, f64)> = None;

  let mut pressure_pos: PositionList = vec![];
  let mut last_pressure_pos: Option<(f64, f64, f64, f64)> = None;

  let mut flow_pos: PositionList = vec![];
  let mut last_flow_pos: Option<(f64, f64, f64, f64)> = None;

  let mut elapsed_time = 0f64;
  let mut prev_pump = None;
  let mut prev_exit_flow: Option<f32> = None;

  for step in steps.iter() {
    let duration = step.seconds() as f64;
    let transition = step.transition();
    let pump = step.pump();

    for prop in step.0.iter() {
      match prop {
        Prop::Temperature(t) => {
          let t = *t as f64;
          if let Some((.., prev_t)) = last_temperature_pos {
            temperature_pos.push((elapsed_time, prev_t, elapsed_time, t));
            temperature_pos.push((elapsed_time, t, elapsed_time + duration, t));
          } else {
            temperature_pos.push((elapsed_time, t, elapsed_time + duration, t));
          }
          last_temperature_pos = Some(temperature_pos.last().unwrap().clone());
        }
        Prop::Pressure(v) => {
          if pump == PumpType::Pressure {
            if let (Some(PumpType::Flow), Some((.., px, py))) = (prev_pump, last_flow_pos) {
              flow_pos.push((px, py, px, 0.));
              last_flow_pos = Some(flow_pos.last().unwrap().clone());
            }

            let v = *v as f64;
            if let Some((.., prev_v)) = last_pressure_pos {
              match transition {
                TransitionType::Fast => {
                  pressure_pos.push((elapsed_time, prev_v, elapsed_time, v));
                  pressure_pos.push((elapsed_time, v, elapsed_time + duration, v));
                }
                TransitionType::Smooth => {
                  pressure_pos.push((elapsed_time, prev_v, elapsed_time + duration, v));
                }
              }
            } else {
              pressure_pos.push((elapsed_time, 0., elapsed_time, v));
              pressure_pos.push((elapsed_time, v, elapsed_time + duration, v));
            }

            last_pressure_pos = Some(pressure_pos.last().unwrap().clone());
          }
        }
        Prop::Flow(v) => {
          if pump == PumpType::Flow {
            if let (Some(PumpType::Pressure), Some((.., px, py))) = (prev_pump, last_pressure_pos) {
              pressure_pos.push((px, py, px, 0.));
              last_pressure_pos = Some(pressure_pos.last().unwrap().clone());
            }

            let v = *v as f64;
            if let Some((.., prev_v)) = last_flow_pos {
              let mut prev_v = prev_v;
              if let Some(f) = prev_exit_flow {
                flow_pos.push((elapsed_time, prev_v, elapsed_time, f as f64));
                prev_v = f as f64;
              }

              match transition {
                TransitionType::Fast => {
                  flow_pos.push((elapsed_time, prev_v, elapsed_time, v));
                  flow_pos.push((elapsed_time, v, elapsed_time + duration, v));
                }
                TransitionType::Smooth => {
                  flow_pos.push((elapsed_time, prev_v, elapsed_time + duration, v));
                }
              }
            } else {
              flow_pos.push((elapsed_time, 0., elapsed_time, v));
              flow_pos.push((elapsed_time, v, elapsed_time + duration, v));
            }

            last_flow_pos = Some(flow_pos.last().unwrap().clone());
          }
        }
        _ => (),
      }
    }

    elapsed_time += duration;
    prev_pump = Some(pump);
    prev_exit_flow = step.exit_flow();
  }

  (temperature_pos, pressure_pos, flow_pos, elapsed_time)
}
