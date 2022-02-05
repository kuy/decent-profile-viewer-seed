use once_cell::sync::Lazy;
use seed::prelude::*;
use seed::*;

use crate::axis::{Axis, Direction};
use crate::msg::Msg;
use crate::parser::Step;
use crate::profile::{analyze, PositionList};
use crate::scale::scale;

static OUTER: Lazy<(f64, f64)> = Lazy::new(|| (600., 400.));
static INNER: Lazy<(f64, f64, f64, f64)> = Lazy::new(|| (30., 20., 580., 370.));

pub fn view_svg(steps: &Vec<Step>) -> Node<Msg> {
  let (temperature_pos, pressure_pos, flow_pos, elapsed_time) = analyze(steps);
  svg![
    attrs![
        At::Width => px(OUTER.0),
        At::Height => px(OUTER.1),
        At::ViewBox => format!("0 0 {} {}", OUTER.0, OUTER.1),
    ],
    view_axis(elapsed_time),
    g![
      view_graph_temperature(&temperature_pos, elapsed_time),
      view_graph_pressure(&pressure_pos, elapsed_time),
      view_graph_flow(&flow_pos, elapsed_time),
    ]
  ]
}

fn view_axis(elapsed_time: f64) -> Node<Msg> {
  let x_axis = Axis::new(
    (0., elapsed_time),
    (0., INNER.2 - INNER.0),
    Direction::Horizontal,
    10.0,
  );
  let y_axis = Axis::new((0., 12.), (0., INNER.1 - INNER.3), Direction::Vertical, 1.0);
  g![
    g![
      attrs![At::Transform => format!("translate({},{})", INNER.0, INNER.3)],
      x_axis.render(),
    ],
    g![
      attrs![At::Transform => format!("translate({},{})", INNER.0, INNER.3)],
      y_axis.render(),
    ],
  ]
}

fn view_graph_temperature(list: &PositionList, elapsed_time: f64) -> Node<Msg> {
  let x = scale((0., elapsed_time), (INNER.0, INNER.2));
  let y = scale((20., 100.), (INNER.3, INNER.1));
  g![list.iter().map(|(x1, y1, x2, y2)| line_![attrs![
    At::X1 => x(*x1),
    At::Y1 => y(*y1),
    At::X2 => x(*x2),
    At::Y2 => y(*y2),
    At::Stroke => "darkred",
    At::StrokeWidth => "1.5px",
    At::StrokeLinecap => "round",
  ]])]
}

fn view_graph_pressure(list: &PositionList, elapsed_time: f64) -> Node<Msg> {
  let x = scale((0., elapsed_time), (INNER.0, INNER.2));
  let y = scale((0., 12.), (INNER.3, INNER.1));
  g![list.iter().map(|(x1, y1, x2, y2)| line_![attrs![
    At::X1 => x(*x1),
    At::Y1 => y(*y1),
    At::X2 => x(*x2),
    At::Y2 => y(*y2),
    At::Stroke => "darkgreen",
    At::StrokeWidth => "1.5px",
    At::StrokeLinecap => "round",
  ]])]
}

fn view_graph_flow(list: &PositionList, elapsed_time: f64) -> Node<Msg> {
  let x = scale((0., elapsed_time), (INNER.0, INNER.2));
  let y = scale((0., 12.), (INNER.3, INNER.1));
  g![list.iter().map(|(x1, y1, x2, y2)| line_![attrs![
    At::X1 => x(*x1),
    At::Y1 => y(*y1),
    At::X2 => x(*x2),
    At::Y2 => y(*y2),
    At::Stroke => "darkblue",
    At::StrokeWidth => "1.5px",
    At::StrokeLinecap => "round",
  ]])]
}
