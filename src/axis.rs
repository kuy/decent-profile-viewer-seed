use seed::prelude::*;
use seed::*;

use crate::msg::Msg;
use crate::scale::scale;

pub struct Axis {
  domain: (f64, f64),
  range: (f64, f64),
  dir: Direction,
  min_unit: f64,
}

pub enum Direction {
  Horizontal,
  Vertical,
}

impl Axis {
  pub fn new(domain: (f64, f64), range: (f64, f64), dir: Direction, min_unit: f64) -> Self {
    Self {
      domain,
      range,
      dir,
      min_unit,
    }
  }

  pub fn render(&self) -> Node<Msg> {
    match self.dir {
      Direction::Horizontal => {
        g![
          line_![attrs![
            At::X1 => 0.,
            At::Y1 => 0.,
            At::X2 => self.range.1,
            At::Y2 => 0.,
            At::Stroke => "darkgray",
            At::StrokeWidth => "1.25px",
            At::StrokeLinecap => "round",
          ]],
          self.render_marks(),
        ]
      }
      Direction::Vertical => {
        g![
          line_![attrs![
            At::X1 => 0.,
            At::Y1 => 0.,
            At::X2 => 0.,
            At::Y2 => self.range.1,
            At::Stroke => "darkgray",
            At::StrokeWidth => "1.25px",
            At::StrokeLinecap => "round",
          ]],
          g![],
        ]
      }
    }
  }

  fn render_marks(&self) -> Node<Msg> {
    match self.dir {
      Direction::Horizontal => {
        let x = scale(self.domain, self.range);
        let mut t = self.min_unit;
        let mut list = vec![];
        loop {
          list.push(line_![attrs![
            At::X1 => x(t),
            At::Y1 => 0.,
            At::X2 => x(t),
            At::Y2 => 10.,
            At::Stroke => "darkgray",
            At::StrokeWidth => "0.75px",
            At::StrokeLinecap => "round",
          ]]);

          t += self.min_unit;
          if t > self.domain.1 {
            break;
          }
        }
        g![list]
      }
      _ => g![],
    }
  }
}
