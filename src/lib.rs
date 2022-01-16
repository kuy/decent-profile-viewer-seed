// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model { text: "".into() }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
    text: String,
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
            model.text = text;
        }
    }
}

// ------ ------
//     View
// ------ ------

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    div![
        style!{
            St::Display => "flex",
            St::FlexDirection => "row",
        },
        div![
            div![
                &model.text,
                style!{St::WhiteSpace => "pre-wrap"},
            ],
            style!{St::FlexGrow => "1",},
        ],
        div![
            textarea![
                style!{
                    St::Width => "100%",
                    St::Height => "100%",
                },
                input_ev(Ev::Input, Msg::Change),
            ],
            style!{
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
