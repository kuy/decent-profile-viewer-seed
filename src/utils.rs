pub fn console_log(msg: String) {
  seed::prelude::web_sys::console::log_1(&seed::prelude::JsValue::from_str(msg.as_str()));
}
