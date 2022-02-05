use once_cell::sync::Lazy;
use seed::prelude::web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use seed::prelude::*;

use crate::context::TranslatedContext;
use crate::parser::Step;
use crate::profile::analyze;
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

  let (temperature_pos, pressure_pos, flow_pos, elapsed_time) = analyze(steps);

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
  ctx.set_line_width(1.25);
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
