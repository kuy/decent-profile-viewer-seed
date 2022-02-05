use seed::prelude::web_sys::MouseEvent;

pub enum Msg {
  Change(String),
  Rendered,
  Select(String),
  Hover(MouseEvent),
}
