// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use std::str::{self, FromStr};

use seed::{prelude::*, *};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace1;
use nom::character::{
  complete::{space1, u16},
  streaming::digit1 as digit,
};
use nom::combinator::{map, map_res, opt, peek, recognize};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
  Model {
    text: "".into(),
    value: 0.0,
    step: Step(vec![]),
  }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
  text: String,
  value: f32,
  step: Step,
}

struct Step(Vec<KeyValue>);
struct Steps(Vec<Step>);

struct KeyValue {
  key: String,
  value: String,
}

// ------ ------
//    Update
// ------ ------

// `Msg` describes the different events you can modify state with.
enum Msg {
  Change(String),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
  match msg {
    Msg::Change(text) => {
      model.text = text.clone();
      let (_, v) = float(text.as_bytes()).unwrap();
      model.value = v;
    }
  }
}

fn unsigned_float(i: &[u8]) -> IResult<&[u8], f32> {
  let float_bytes = recognize(alt((
    delimited(digit, tag("."), opt(digit)),
    delimited(opt(digit), tag("."), digit),
  )));
  let float_str = map_res(float_bytes, str::from_utf8);
  map_res(float_str, FromStr::from_str)(i)
}

fn float(i: &[u8]) -> IResult<&[u8], f32> {
  map(
    pair(opt(alt((tag("+"), tag("-")))), unsigned_float),
    |(sign, value)| {
      sign
        .and_then(|s| if s[0] == b'-' { Some(-1f32) } else { None })
        .unwrap_or(1f32)
        * value
    },
  )(i)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PropName {
  ExitIf,
  Flow,
  Unknown,
}

impl From<&[u8]> for PropName {
  fn from(value: &[u8]) -> Self {
    match value {
      b"exit_if" => PropName::ExitIf,
      b"flow" => PropName::Flow,
      _ => PropName::Unknown,
    }
  }
}

fn prop_name(i: &[u8]) -> IResult<&[u8], PropName> {
  let (i, name) = alt((tag("exit_if"), tag("flow")))(i)?;
  Ok((i, PropName::from(name)))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Prop {
  ExitIf(bool),
  Flow(f32),
  Volume(u16),
  MaxFlowOrPressureRange(f32),
  ExitFlowUnder(f32),
  Temperature(f32),
  Pressure(f32),
  ExitFlowOver(f32),
  ExitPressureOver(f32),
  MaxFlowOrPressure(f32),
  ExitPressureUnder(f32),
  Seconds(f32),
  Unknown,
}

fn bool_val(i: &[u8]) -> IResult<&[u8], bool> {
  let (i, v) = alt((tag("0"), tag("1")))(i)?;
  Ok((i, v == &b"1"[..]))
}

fn number_val(i: &[u8]) -> IResult<&[u8], f32> {
  match peek(unsigned_float)(i) {
    Ok((i, _)) => unsigned_float(i),
    _ => map(u16, |v| v as f32)(i),
  }
}

fn prop_bool(name: &str) -> impl Fn(&[u8]) -> IResult<&[u8], Prop> {
  let name = name.to_string();
  move |i: &[u8]| {
    let (i, (_, _, val)) = tuple((tag(name.as_bytes()), space1, bool_val))(i)?;
    let prop = match name.as_str() {
      "exit_if" => Prop::ExitIf(val),
      _ => Prop::Unknown,
    };
    Ok((i, prop))
  }
}

fn prop_int(name: &str) -> impl Fn(&[u8]) -> IResult<&[u8], Prop> {
  let name = name.to_string();
  move |i: &[u8]| {
    let (i, (_, _, val)) = tuple((tag(name.as_bytes()), space1, u16))(i)?;
    let prop = match name.as_str() {
      "volume" => Prop::Volume(val),
      _ => Prop::Unknown,
    };
    Ok((i, prop))
  }
}

fn prop_number(name: &str) -> impl Fn(&[u8]) -> IResult<&[u8], Prop> {
  let name = name.to_string();
  move |i: &[u8]| {
    let (i, (_, _, val)) = tuple((tag(name.as_bytes()), space1, number_val))(i)?;
    let prop = match name.as_str() {
      "flow" => Prop::Flow(val),
      "max_flow_or_pressure_range" => Prop::MaxFlowOrPressureRange(val),
      "exit_flow_under" => Prop::MaxFlowOrPressureRange(val),
      "temperature" => Prop::Temperature(val),
      "pressure" => Prop::Pressure(val),
      "exit_flow_over" => Prop::ExitFlowOver(val),
      "exit_pressure_over" => Prop::ExitPressureOver(val),
      "max_flow_or_pressure" => Prop::MaxFlowOrPressure(val),
      "exit_pressure_under" => Prop::ExitPressureUnder(val),
      "seconds" => Prop::Seconds(val),
      _ => Prop::Unknown,
    };
    Ok((i, prop))
  }
}

fn prop(i: &[u8]) -> IResult<&[u8], Prop> {
  alt((
    prop_bool("exit_if"),
    prop_number("flow"),
    prop_int("volume"),
    prop_number("max_flow_or_pressure_range"),
    prop_number("exit_flow_under"),
    prop_number("temperature"),
    prop_number("pressure"),
    prop_number("exit_flow_over"),
    prop_number("exit_pressure_over"),
    prop_number("max_flow_or_pressure"),
    prop_number("exit_pressure_under"),
    prop_number("seconds"),
  ))(i)
}

fn props(i: &[u8]) -> IResult<&[u8], Vec<Prop>> {
  separated_list0(multispace1, prop)(i)
}

// fn category(i: &[u8]) -> IResult<&[u8], &str> {
//   map_res(
//     delimited(char('['), take_while(|c| c != b']'), char(']')),
//     str::from_utf8,
//   )(i)
// }

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
      div![format!("{}", model.value),],
      div![&model.text, style! {St::WhiteSpace => "pre-wrap"},],
      style! {St::FlexGrow => "1",},
    ],
    div![
      textarea![
        style! {
            St::Width => "100%",
            St::Height => "100%",
        },
        input_ev(Ev::Input, Msg::Change),
      ],
      style! {
          St::FlexGrow => "1",
          St::MinHeight => "400px",
      },
    ],
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

#[cfg(test)]
mod tests {
  use nom::error::{Error, ErrorKind};

  use super::*;

  #[test]
  fn test_number_val() {
    assert_eq!(number_val(b"8;"), Ok((&b";"[..], 8.0f32)));
    assert_eq!(number_val(b"80;"), Ok((&b";"[..], 80.0f32)));
    assert_eq!(number_val(b"8.;"), Ok((&b";"[..], 8.0f32)));
    assert_eq!(number_val(b"8.0;"), Ok((&b";"[..], 8.0f32)));
    assert_eq!(number_val(b".8;"), Ok((&b";"[..], 0.8f32)));
  }

  #[test]
  fn test_prop_name() {
    assert_eq!(prop_name(b"exit_if;"), Ok((&b";"[..], PropName::ExitIf)));
  }

  #[test]
  fn test_prop_bool() {
    let prop_exit_if = prop_bool("exit_if");
    assert_eq!(
      prop_exit_if(b"exit_if 1;"),
      Ok((&b";"[..], Prop::ExitIf(true)))
    );
    assert_eq!(
      prop_exit_if(b"exit_if 0;"),
      Ok((&b";"[..], Prop::ExitIf(false)))
    );
    assert_eq!(
      prop_exit_if(b"exit_if x;"),
      Err(nom::Err::Error(Error::new(&b"x;"[..], ErrorKind::Tag)))
    );
  }

  #[test]
  fn test_prop_int() {
    let prop_flow = prop_int("flow");
    assert_eq!(prop_flow(b"flow 8;"), Ok((&b";"[..], Prop::Flow(8.0))));
    assert_eq!(
      prop_flow(b"flow -1;"),
      Err(nom::Err::Error(Error::new(&b"-1;"[..], ErrorKind::Digit)))
    );

    let prop_volume = prop_int("volume");
    assert_eq!(
      prop_volume(b"volume 100;"),
      Ok((&b";"[..], Prop::Volume(100)))
    );
    assert_eq!(
      prop_volume(b"volume x;"),
      Err(nom::Err::Error(Error::new(&b"x;"[..], ErrorKind::Digit)))
    );
  }

  #[test]
  fn test_prop() {
    assert_eq!(prop(b"flow 8;"), Ok((&b";"[..], Prop::Flow(8.0))));
    assert_eq!(prop(b"volume 100;"), Ok((&b";"[..], Prop::Volume(100))));
    assert_eq!(
      prop(b"exit_pressure_over 1.5;"),
      Ok((&b";"[..], Prop::ExitPressureOver(1.5)))
    );
  }

  #[test]
  fn test_props() {
    assert_eq!(props(b";"), Ok((&b";"[..], vec![])));
    assert_eq!(props(b"flow 8;"), Ok((&b";"[..], vec![Prop::Flow(8.0)])));
    assert_eq!(
      props(b"volume 100 seconds 25.00;"),
      Ok((&b";"[..], vec![Prop::Volume(100), Prop::Seconds(25.0)]))
    );
    assert_eq!(
      props(b"volume 100\nseconds 127\nexit_if 0;"),
      Ok((
        &b";"[..],
        vec![Prop::Volume(100), Prop::Seconds(127.0), Prop::ExitIf(false)]
      ))
    );
  }

  #[test]
  fn test_simple() {
    let tcl = include_str!("../fixtures/simple.tcl");
    // assert_eq!(tcl, "hoge");
  }
}
