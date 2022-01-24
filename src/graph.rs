use once_cell::sync::Lazy;
use seed::prelude::web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use seed::prelude::*;

use crate::context::TranslatedContext;
use crate::parser::{Prop, PumpType, Step, TransitionType};
use crate::scale::scale;

static CANVAS: Lazy<(f64, f64)> = Lazy::new(|| (600., 400.));
static INNER: Lazy<(f64, f64, f64, f64)> = Lazy::new(|| (30., 20., 580., 370.));

pub fn draw(canvas: &ElRef<HtmlCanvasElement>, steps: &Vec<Step>) {
  let canvas = canvas.get().expect("should get canvas");
  let ctx = seed::canvas_context_2d(&canvas);

  // clear canvas
  ctx.begin_path();
  ctx.clear_rect(0., 0., CANVAS.0, CANVAS.1);

  draw_axis(&ctx);

  // analyze profile
  let mut temperature_pos: Vec<(f64, f64, f64, f64)> = vec![];
  let mut pressure_pos: Vec<(f64, f64, f64, f64)> = vec![];
  let mut flow_pos: Vec<(f64, f64, f64, f64)> = vec![];

  let mut elapsed_time = 0f64;
  let mut prev_pump = None;

  steps.iter().for_each(|step| {
    let duration = step.seconds() as f64;
    let transition = step.transition();
    let pump = step.pump();

    step.0.iter().for_each(|prop| match prop {
      Prop::Temperature(t) => {
        let t = *t as f64;
        if let Some((.., prev_t)) = temperature_pos.last() {
          temperature_pos.push((elapsed_time, *prev_t, elapsed_time, t));
          temperature_pos.push((elapsed_time, t, elapsed_time + duration, t));
        } else {
          temperature_pos.push((elapsed_time, t, elapsed_time + duration, t));
        }
      }
      Prop::Pressure(v) => {
        if pump == PumpType::Pressure {
          if let (Some(PumpType::Flow), Some((.., px, py))) = (prev_pump, flow_pos.last()) {
            flow_pos.push((*px, *py, *px, 0.));
          }

          let v = *v as f64;
          if let Some((.., prev_v)) = pressure_pos.last() {
            match transition {
              TransitionType::Fast => {
                pressure_pos.push((elapsed_time, *prev_v, elapsed_time, v));
                pressure_pos.push((elapsed_time, v, elapsed_time + duration, v));
              }
              TransitionType::Smooth => {
                pressure_pos.push((elapsed_time, *prev_v, elapsed_time + duration, v));
              }
            }
          } else {
            pressure_pos.push((elapsed_time, 0., elapsed_time, v));
            pressure_pos.push((elapsed_time, v, elapsed_time + duration, v));
          }
        }
      }
      Prop::Flow(v) => {
        if pump == PumpType::Flow {
          if let (Some(PumpType::Pressure), Some((.., px, py))) = (prev_pump, pressure_pos.last()) {
            pressure_pos.push((*px, *py, *px, 0.));
          }

          let v = *v as f64;
          if let Some((.., prev_v)) = flow_pos.last() {
            match transition {
              TransitionType::Fast => {
                flow_pos.push((elapsed_time, *prev_v, elapsed_time, v));
                flow_pos.push((elapsed_time, v, elapsed_time + duration, v));
              }
              TransitionType::Smooth => {
                flow_pos.push((elapsed_time, *prev_v, elapsed_time + duration, v));
              }
            }
          } else {
            flow_pos.push((elapsed_time, 0., elapsed_time, v));
            flow_pos.push((elapsed_time, v, elapsed_time + duration, v));
          }
        }
      }
      _ => (),
    });

    elapsed_time += duration;
    prev_pump = Some(pump);
  });

  let temp_ctx = TranslatedContext::new(
    &ctx,
    Box::new(scale((0., elapsed_time), (INNER.0, INNER.2))),
    Box::new(scale((20., 100.), (INNER.3, INNER.1))),
  );

  let pressure_ctx = TranslatedContext::new(
    &ctx,
    Box::new(scale((0., elapsed_time), (INNER.0, INNER.2))),
    Box::new(scale((0., 12.), (INNER.3, INNER.1))),
  );

  let flow_ctx = TranslatedContext::new(
    &ctx,
    Box::new(scale((0., elapsed_time), (INNER.0, INNER.2))),
    Box::new(scale((0., 12.), (INNER.3, INNER.1))),
  );

  // draw profile
  temperature_pos.iter().for_each(|(x1, y1, x2, y2)| {
    temp_ctx.line(*x1, *y1, *x2, *y2, "darkred");
  });

  pressure_pos.iter().for_each(|(x1, y1, x2, y2)| {
    pressure_ctx.line(*x1, *y1, *x2, *y2, "darkgreen");
  });

  flow_pos.iter().for_each(|(x1, y1, x2, y2)| {
    flow_ctx.line(*x1, *y1, *x2, *y2, "darkblue");
  });
}

fn draw_axis(ctx: &CanvasRenderingContext2d) {
  ctx.begin_path();
  ctx.set_stroke_style(&JsValue::from_str("gray"));

  // x-axis
  ctx.move_to(INNER.0, INNER.3);
  ctx.line_to(INNER.2, INNER.3);
  ctx.stroke();

  // y-axis
  ctx.move_to(INNER.0, INNER.3);
  ctx.line_to(INNER.0, INNER.1);
  ctx.stroke();
}
