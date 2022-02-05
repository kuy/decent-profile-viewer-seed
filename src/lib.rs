// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

mod msg;
mod parser;
mod profile;
mod scale;
mod utils;
mod view;

use seed::prelude::*;
use seed::*;

use msg::Msg;
use parser::{steps, Step};
use profile::PROFILES;
use view::view_svg;

use crate::profile::Preset;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
  Model {
    text: "".into(),
    steps: vec![],
    error: false,
    selected: None,
  }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
  text: String,
  steps: Vec<Step>,
  error: bool,
  selected: Option<String>,
}

// ------ ------
//    Update
// ------ ------

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
  match msg {
    Msg::Change(text) => {
      model.text = text.clone();
      match steps(text.as_bytes()) {
        Ok((_, steps)) => {
          model.steps = steps;
          model.error = false;
        }
        _ => model.error = true,
      }
    }
    Msg::Select(file_name) => {
      model.selected = Some(file_name.clone());

      let data = PROFILES.get(&file_name).expect("should exist").data.clone();
      orders.send_msg(Msg::Change(data));
    }
  }
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
  div![
    style! {
        St::Display => "flex",
        St::FlexDirection => "row",
    },
    div![
      div![view_svg(&model.steps)],
      div![view_syntax_error(model.error)],
      div![model.steps.iter().map(|step| view_step(step))],
      hr![],
      div![&model.text, style! {St::WhiteSpace => "pre-wrap"},],
      style! {St::Flex => "1 1 0px",},
    ],
    div![
      div![view_profile_selector()],
      textarea![
        attrs! {
          At::Value => model.text.clone(),
        },
        style! {
            St::Width => "100%",
            St::Height => "100%",
        },
        input_ev(Ev::Input, Msg::Change),
      ],
      style! {
          St::Flex => "1 1 0px",
          St::MinHeight => "400px",
      },
    ],
  ]
}

fn view_syntax_error(error: bool) -> Vec<Node<Msg>> {
  let mut children = vec![];
  if error {
    children.push(span![
      "Syntax Error",
      style! {
        St::Color => "red",
        St::FontWeight => "bold",
      }
    ]);
  }
  children
}

fn view_step(step: &Step) -> Node<Msg> {
  div![
    step.0.iter().map(|prop| div![format!("{:?}", prop),]),
    style! { St::Border => "1px solid black" }
  ]
}

fn view_profile_selector() -> Node<Msg> {
  let mut presets = PROFILES
    .iter()
    .map(|(file_name, preset)| (file_name.clone(), preset.clone()))
    .collect::<Vec<(String, Preset)>>();
  presets.sort_by(|(_, p1), (_, p2)| p1.title.cmp(&p2.title));

  select![
    option!["--- select profile ---"],
    presets
      .iter()
      .map(|(file_name, preset)| option![attrs! { At::Value => file_name }, preset.title.as_str()]),
    input_ev(Ev::Change, Msg::Select)
  ]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
  // Mount the `app` to the element with the `id` "app".
  App::start("app", init, update, view);
}
