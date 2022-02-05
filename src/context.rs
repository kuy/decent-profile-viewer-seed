use seed::prelude::{web_sys::CanvasRenderingContext2d, JsValue};

pub struct TranslatedContext {
  ctx: CanvasRenderingContext2d,
  x: Box<dyn Fn(f64) -> f64>,
  y: Box<dyn Fn(f64) -> f64>,
}

impl TranslatedContext {
  pub fn new(
    ctx: &CanvasRenderingContext2d,
    x: Box<dyn Fn(f64) -> f64>,
    y: Box<dyn Fn(f64) -> f64>,
  ) -> Self {
    Self {
      ctx: ctx.clone(),
      x,
      y,
    }
  }

  pub fn line(&self, x1: f64, y1: f64, x2: f64, y2: f64, color: &str) {
    self.ctx.begin_path();
    self.ctx.set_line_width(1.25);
    self.ctx.set_stroke_style(&JsValue::from_str(color));

    let (tx1, ty1) = self.translate(x1, y1);
    self.ctx.move_to(tx1, ty1);

    let (tx2, ty2) = self.translate(x2, y2);
    self.ctx.line_to(tx2, ty2);

    self.ctx.stroke();
  }

  fn translate(&self, x: f64, y: f64) -> (f64, f64) {
    ((self.x)(x), (self.y)(y))
  }
}
