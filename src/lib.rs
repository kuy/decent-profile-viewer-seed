// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use std::convert::TryFrom;
use std::str::{self, FromStr};

use seed::{prelude::*, *};

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until};
use nom::character::complete::{multispace0, multispace1};
use nom::character::{
  complete::{space1, u16},
  is_newline, is_space,
  streaming::digit1 as digit,
};
use nom::combinator::{map, map_res, opt, peek, recognize};
use nom::multi::separated_list0;
use nom::sequence::{delimited, tuple};
use nom::IResult;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
  Model {
    text: "".into(),
    steps: vec![],
    error: false,
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
      match steps(text.as_bytes()) {
        Ok((_, steps)) => {
          model.steps = steps;
          model.error = false;
        }
        _ => model.error = true,
      }
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

#[derive(Clone, Debug, PartialEq)]
enum Prop {
  ExitIf(bool),
  Flow(f32),
  Volume(u16),
  MaxFlowOrPressureRange(f32),
  Transition(TransitionType),
  ExitFlowUnder(f32),
  Temperature(f32),
  Name(String),
  Pressure(f32),
  Sensor(SensorType),
  Pump(PumpType),
  ExitType(ExitType),
  ExitFlowOver(f32),
  ExitPressureOver(f32),
  MaxFlowOrPressure(f32),
  ExitPressureUnder(f32),
  Seconds(f32),
  Unknown,
}

#[derive(Clone, Debug)]
struct ConvertError(String);

impl TryFrom<TransitionType> for Prop {
  type Error = ConvertError;

  fn try_from(value: TransitionType) -> Result<Self, Self::Error> {
    Ok(Prop::Transition(value))
  }
}

trait ParsableEnumProp {
  fn parse(i: &[u8]) -> IResult<&[u8], Prop>;
}

#[derive(Clone, Debug)]
struct UnexpectedValueError(String);

#[derive(Clone, Copy, Debug, PartialEq)]
enum TransitionType {
  Fast,
  Smooth,
}

impl TryFrom<&[u8]> for TransitionType {
  type Error = UnexpectedValueError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let value = str::from_utf8(value).expect("should be converted");
    let ret = match value {
      "fast" => TransitionType::Fast,
      "smooth" => TransitionType::Smooth,
      _ => return Err(UnexpectedValueError(value.into())),
    };
    Ok(ret)
  }
}

impl ParsableEnumProp for TransitionType {
  fn parse(i: &[u8]) -> IResult<&[u8], Prop> {
    let (i, (_, _, val)) = tuple((tag("transition"), space1, transition_val))(i)?;
    Ok((i, Prop::Transition(val)))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SensorType {
  Coffee,
  Water,
}

impl TryFrom<&[u8]> for SensorType {
  type Error = UnexpectedValueError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let value = str::from_utf8(value).expect("should be converted");
    let ret = match value {
      "coffee" => SensorType::Coffee,
      "water" => SensorType::Water,
      _ => return Err(UnexpectedValueError(value.into())),
    };
    Ok(ret)
  }
}

impl ParsableEnumProp for SensorType {
  fn parse(i: &[u8]) -> IResult<&[u8], Prop> {
    let (i, (_, _, val)) = tuple((tag("sensor"), space1, sensor_val))(i)?;
    Ok((i, Prop::Sensor(val)))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PumpType {
  Flow,
  Pressure,
}

impl TryFrom<&[u8]> for PumpType {
  type Error = UnexpectedValueError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let value = str::from_utf8(value).expect("should be converted");
    let ret = match value {
      "flow" => PumpType::Flow,
      "pressure" => PumpType::Pressure,
      _ => return Err(UnexpectedValueError(value.into())),
    };
    Ok(ret)
  }
}

impl ParsableEnumProp for PumpType {
  fn parse(i: &[u8]) -> IResult<&[u8], Prop> {
    let (i, (_, _, val)) = tuple((tag("pump"), space1, pump_val))(i)?;
    Ok((i, Prop::Pump(val)))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ExitType {
  PressureUnder,
  PressureOver,
  FlowUnder,
  FlowOver,
}

impl TryFrom<&[u8]> for ExitType {
  type Error = UnexpectedValueError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let value = str::from_utf8(value).expect("should be converted");
    let ret = match value {
      "pressure_under" => ExitType::PressureUnder,
      "pressure_over" => ExitType::PressureOver,
      "flow_under" => ExitType::FlowUnder,
      "flow_over" => ExitType::FlowOver,
      _ => return Err(UnexpectedValueError(value.into())),
    };
    Ok(ret)
  }
}

#[derive(Clone, Debug, PartialEq)]
struct Step(Vec<Prop>);

impl ParsableEnumProp for ExitType {
  fn parse(i: &[u8]) -> IResult<&[u8], Prop> {
    let (i, (_, _, val)) = tuple((tag("exit_type"), space1, exit_type_val))(i)?;
    Ok((i, Prop::ExitType(val)))
  }
}

fn transition_val(i: &[u8]) -> IResult<&[u8], TransitionType> {
  map_res(alt((tag("fast"), tag("smooth"))), TransitionType::try_from)(i)
}

fn sensor_val(i: &[u8]) -> IResult<&[u8], SensorType> {
  map_res(alt((tag("coffee"), tag("water"))), SensorType::try_from)(i)
}

fn pump_val(i: &[u8]) -> IResult<&[u8], PumpType> {
  map_res(alt((tag("flow"), tag("pressure"))), PumpType::try_from)(i)
}

fn exit_type_val(i: &[u8]) -> IResult<&[u8], ExitType> {
  map_res(
    alt((
      tag("pressure_under"),
      tag("pressure_over"),
      tag("flow_under"),
      tag("flow_over"),
    )),
    ExitType::try_from,
  )(i)
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

fn plain_string_val(i: &[u8]) -> IResult<&[u8], String> {
  let (i, v) = take_till(|c| is_space(c) || is_newline(c))(i)?;
  Ok((
    i,
    String::from_utf8(v.to_vec()).expect("should be converted"),
  ))
}

fn bracket_string_val(i: &[u8]) -> IResult<&[u8], String> {
  let (i, (_, v, _)) = tuple((tag("{"), take_until("}"), tag("}")))(i)?;
  Ok((
    i,
    String::from_utf8(v.to_vec()).expect("should be converted"),
  ))
}

fn string_val(i: &[u8]) -> IResult<&[u8], String> {
  alt((bracket_string_val, plain_string_val))(i)
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
      "exit_flow_under" => Prop::ExitFlowUnder(val),
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

fn prop_enum<E>() -> impl Fn(&[u8]) -> IResult<&[u8], Prop>
where
  E: ParsableEnumProp,
{
  |i: &[u8]| E::parse(i)
}

fn prop_string(name: &str) -> impl Fn(&[u8]) -> IResult<&[u8], Prop> {
  let name = name.to_string();
  move |i: &[u8]| {
    let (i, (_, _, val)) = tuple((tag(name.as_bytes()), space1, string_val))(i)?;
    let prop = match name.as_str() {
      "name" => Prop::Name(val),
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
    prop_enum::<TransitionType>(),
    prop_number("exit_flow_under"),
    prop_number("temperature"),
    prop_string("name"),
    prop_number("pressure"),
    prop_enum::<SensorType>(),
    prop_enum::<PumpType>(),
    prop_enum::<ExitType>(),
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

fn step(i: &[u8]) -> IResult<&[u8], Step> {
  let (i, (_, _, v, _, _)) = tuple((tag("{"), multispace0, props, multispace0, tag("}")))(i)?;
  Ok((i, Step(v)))
}

fn steps(i: &[u8]) -> IResult<&[u8], Vec<Step>> {
  separated_list0(multispace0, step)(i)
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
      div![view_syntax_error(model.error)],
      div![model.steps.iter().map(|step| view_step(step))],
      hr![],
      div![&model.text, style! {St::WhiteSpace => "pre-wrap"},],
      style! {St::Flex => "1 1 0px",},
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
  fn test_string_val() {
    assert_eq!(string_val(b"Fill ;"), Ok((&b" ;"[..], "Fill".into())));
    assert_eq!(string_val(b"Fill\n;"), Ok((&b"\n;"[..], "Fill".into())));
    assert_eq!(
      string_val(b"{Pressure Up};"),
      Ok((&b";"[..], "Pressure Up".into()))
    );
    assert_eq!(
      string_val(b"{New\n\"Line\"\n\nSupported \n};"),
      Ok((&b";"[..], "New\n\"Line\"\n\nSupported \n".into()))
    );
  }

  #[test]
  fn test_transition_val() {
    assert_eq!(
      transition_val(b"fast;"),
      Ok((&b";"[..], TransitionType::Fast))
    );
    assert_eq!(
      transition_val(b"smooth;"),
      Ok((&b";"[..], TransitionType::Smooth))
    );
    assert_eq!(
      transition_val(b"slow;"),
      Err(nom::Err::Error(Error::new(&b"slow;"[..], ErrorKind::Tag)))
    );
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
  fn test_prop_enum() {
    let prop_transition = prop_enum::<TransitionType>();
    assert_eq!(
      prop_transition(b"transition fast;"),
      Ok((&b";"[..], Prop::Transition(TransitionType::Fast)))
    );
    assert_eq!(
      prop_transition(b"transition smooth;"),
      Ok((&b";"[..], Prop::Transition(TransitionType::Smooth)))
    );
    assert_eq!(
      prop_transition(b"transition smooooch;"),
      Err(nom::Err::Error(Error::new(
        &b"smooooch;"[..],
        ErrorKind::Tag
      )))
    );
  }

  #[test]
  fn test_prop_string() {
    let prop_name = prop_string("name");
    assert_eq!(
      prop_name(b"name Fill\n"),
      Ok((&b"\n"[..], Prop::Name("Fill".into())))
    );
    assert_eq!(
      prop_name(b"name {Pressure Up}\n"),
      Ok((&b"\n"[..], Prop::Name("Pressure Up".into())))
    );
    assert_eq!(
      prop_name(b"name {}\n"),
      Ok((&b"\n"[..], Prop::Name("".into())))
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
  fn test_step_inner() {
    let tcl = include_str!("../fixtures/step.inner");
    assert_eq!(
      props(tcl.as_bytes()),
      Ok((
        &b".00"[..],
        vec![
          Prop::ExitIf(true),
          Prop::Flow(8.0),
          Prop::Volume(100),
          Prop::MaxFlowOrPressureRange(0.6),
          Prop::Transition(TransitionType::Fast),
          Prop::ExitFlowUnder(0.0),
          Prop::Temperature(94.0),
          Prop::Name("Fill".into()),
          Prop::Pressure(2.0),
          Prop::Sensor(SensorType::Coffee),
          Prop::Pump(PumpType::Pressure),
          Prop::ExitType(ExitType::PressureOver),
          Prop::ExitFlowOver(6.0),
          Prop::ExitPressureOver(1.5),
          Prop::MaxFlowOrPressure(0.0),
          Prop::ExitPressureUnder(0.0),
          Prop::Seconds(25.0),
        ]
      ))
    );
  }

  #[test]
  fn test_step_outer() {
    let tcl = include_str!("../fixtures/step.outer");
    assert_eq!(
      step(tcl.as_bytes()),
      Ok((
        &b""[..],
        Step(vec![
          Prop::ExitIf(true),
          Prop::Flow(8.0),
          Prop::Volume(100),
          Prop::MaxFlowOrPressureRange(0.6),
          Prop::Transition(TransitionType::Fast),
          Prop::ExitFlowUnder(0.0),
          Prop::Temperature(94.0),
          Prop::Name("Fill".into()),
          Prop::Pressure(2.0),
          Prop::Sensor(SensorType::Coffee),
          Prop::Pump(PumpType::Pressure),
          Prop::ExitType(ExitType::PressureOver),
          Prop::ExitFlowOver(6.0),
          Prop::ExitPressureOver(1.5),
          Prop::MaxFlowOrPressure(0.0),
          Prop::ExitPressureUnder(0.0),
          Prop::Seconds(25.0),
        ])
      ))
    );
  }

  #[test]
  fn test_steps_inner() {
    assert_eq!(
      steps(&b"{volume 100}\n{flow 8}\n"[..]),
      Ok((
        &b"\n"[..],
        vec![Step(vec![Prop::Volume(100),]), Step(vec![Prop::Flow(8.0)])]
      ))
    );

    let tcl = include_str!("../fixtures/steps.inner");
    assert_eq!(
      steps(tcl.as_bytes()),
      Ok((
        &b"\n"[..],
        vec![
          Step(vec![
            Prop::ExitIf(true),
            Prop::Flow(8.0),
            Prop::Volume(100),
            Prop::MaxFlowOrPressureRange(0.6),
            Prop::Transition(TransitionType::Fast),
            Prop::ExitFlowUnder(0.0),
            Prop::Temperature(94.0),
            Prop::Name("Fill".into()),
            Prop::Pressure(2.0),
            Prop::Sensor(SensorType::Coffee),
            Prop::Pump(PumpType::Pressure),
            Prop::ExitType(ExitType::PressureOver),
            Prop::ExitFlowOver(6.0),
            Prop::ExitPressureOver(1.5),
            Prop::MaxFlowOrPressure(0.0),
            Prop::ExitPressureUnder(0.0),
            Prop::Seconds(25.0),
          ]),
          Step(vec![
            Prop::ExitIf(false),
            Prop::Volume(100),
            Prop::MaxFlowOrPressureRange(0.6),
            Prop::Transition(TransitionType::Fast),
            Prop::ExitFlowUnder(0.0),
            Prop::Temperature(93.0),
            Prop::Name("Pressure Up".into()),
            Prop::Pressure(9.0),
            Prop::Sensor(SensorType::Coffee),
            Prop::Pump(PumpType::Pressure),
            Prop::ExitFlowOver(6.0),
            Prop::ExitPressureOver(11.0),
            Prop::MaxFlowOrPressure(0.0),
            Prop::Seconds(4.0),
            Prop::ExitPressureUnder(0.0),
          ])
        ]
      ))
    );
  }
}
