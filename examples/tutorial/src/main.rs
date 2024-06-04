use std::collections::BTreeMap;

use ravel::with;
use ravel_web::{
    any, attr, collections::btree_map, collections::slice, el, event::*,
    format_text, run::spawn_body, text::text, State, View,
};
use web_sys::{
    wasm_bindgen::{JsCast as _, UnwrapThrowExt},
    HtmlInputElement,
};

/// Our model type contains the global state of the application.
struct Model {
    count: usize,
    message: String,
    item_map: BTreeMap<usize, String>,
    item_vec: Vec<(usize, String)>,
}

/// We can build our application modularly out of components. A component is
/// just some function which returns a [`View`]. For now, you can ignore the
/// associated [`State`] constraint.
fn basic_html() -> impl View<State = impl State<Model>> {
    // We can build [`View`]s out of HTML elements, which are defined in the
    // [`el`] module. These take another [`View`] parameter for their body.
    //
    // We can also compose [`View`]s using tuples.
    el::header((
        el::h1(
            // To produce text, we can directly use a [`&'static str`].
            "Ravel tutorial",
        ),
        el::p((
            "This is a basic introduction to ",
            el::a((
                "Ravel",
                // Likewise, HTML attributes defined in the [`attr`] module take
                // their value (typically a string) as a parameter.
                attr::href("https://github.com/kmicklas/ravel"),
            )),
            ".",
        )),
    ))
}

/// Components can take data in parameters, which can be borrowed from shared
/// state such as our [`Model`]. When using borrowed data, we need to add an
/// appropriate bound to the return type for the captured lifetime (here
/// `'_ +`).
fn state(model: &Model) -> impl '_ + View<State = impl State<Model>> {
    (
        el::h2("State"),
        el::p((
            "Count: ",
            // To generate strings dynamically, we can use standard format
            // strings using [`format_text`].
            format_text!("{}", model.count),
        )),
        el::p((
            "Message: ",
            // Previously, we only genereted static strings, which can be used
            // directly. This is also possible for a by-value [`String`].
            //
            // However, for any other string-like type, we need to use [`text`].
            text(&model.message),
        )),
    )
}

/// So far, we've only read data from our model, but have not changed it.
/// Typically, we want to do this in response to events. Now we get to the
/// `State = impl State<Model>` constraint. Using [`Model`] here lets us write
/// event handlers which have mutable access to the data.
///
/// Note that we never want to capture any lifetime parameters on the [`State`]
/// type, which must remain `'static`.
fn events() -> impl View<State = impl State<Model>> {
    (
        el::h2("Events"),
        el::p(el::button((
            "Increment count",
            // We can update the model in response to a chosen HTML event type.
            on_(Click, |model: &mut Model| {
                model.item_map.insert(model.count, model.message.clone());
                model.item_vec.push((model.count, model.message.clone()));
                model.count += 1;
            }),
        ))),
        el::p((
            "Message: ",
            // [`on`], unlike [`on_`], also gives us access to the underlying
            // [`web_sys::Event`].
            el::input(on(InputEvent, |model: &mut Model, event| {
                model.message = event
                    .target()
                    .unwrap_throw()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap_throw()
                    .value();
            })),
        )),
    )
}

/// All of our views so far have had a static structure. Sometimes, we need to
/// swap out or hide various components.
fn dynamic_view(model: &Model) -> impl '_ + View<State = impl State<Model>> {
    (
        el::h2("Dynamic view"),
        // In the general case, we need the following pattern to dynamically
        // select a component type:
        //
        // * Use [`with`] with a closure taking our context `cx`.
        // * Branch according to our chosen logic.
        // * In each branch, return `cx.build(any(...))` for our chosen view.
        el::p(with(|cx| {
            if model.count % 2 == 0 {
                cx.build(any(el::b("Even!")))
            } else {
                cx.build(any("Odd."))
            }
        })),
        // One very common case of a dynamic component is one which is simply
        // present or not. In this case, it is simpler and more efficient to
        // just wrap it in an [`Option`].
        model
            .count
            .is_power_of_two()
            .then(|| el::p("Power of two!")),
    )
}

/// Any non-trivial application will have some dynamically sized list data.
///
/// With a similar structure to our use of [`with`] above, we can generate a
/// [`View`] over a [`BTreeMap`] with [`btree_map()`]. This is useful when the
/// entries are ordered by some type of key, but may be inserted or removed at
/// any position.
///
/// If the data is just an array which grows or shrinks at the end, we can use
/// [`slice()`] to generate a [`View`] over a [`Vec`]/slice.
fn lists(model: &Model) -> impl '_ + View<State = impl State<Model>> {
    (
        el::h2("Map view"),
        el::p(el::table((
            el::thead(el::tr((el::td(()), el::td("Id"), el::td("Message")))),
            el::tbody(btree_map(&model.item_map, |cx, key, value| {
                let key = *key;
                cx.build(el::tr((
                    el::td(el::button((
                        "Remove",
                        on_(Click, {
                            move |model: &mut Model| {
                                model.item_map.remove(&key);
                            }
                        }),
                    ))),
                    el::td(format_text!("{}", key)),
                    el::td(text(value)),
                )))
            })),
        ))),
        el::h2("Slice view"),
        el::p(el::table((
            el::thead(el::tr((el::td(()), el::td("Id"), el::td("Message")))),
            el::tbody(slice(&model.item_vec, |cx, i, (key, value)| {
                let key = *key;
                cx.build(el::tr((
                    el::td(el::button((
                        "Truncate",
                        on_(Click, {
                            move |model: &mut Model| {
                                model.item_vec.truncate(i);
                            }
                        }),
                    ))),
                    el::td(format_text!("{}", key)),
                    el::td(text(value)),
                )))
            })),
        ))),
    )
}

/// Putting it all together...
fn tutorial(model: &Model) -> impl '_ + View<State = impl State<Model>> {
    (
        basic_html(),
        state(model),
        events(),
        dynamic_view(model),
        lists(model),
    )
}

fn main() {
    // Dump any Rust panics to the browser console.
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    // Direct [`log`] lines to the browser console.
    console_log::init_with_level(log::Level::Trace).unwrap();

    spawn_body(
        // Our initial model state:
        Model {
            count: 0,
            message: String::new(),
            item_map: BTreeMap::new(),
            item_vec: Vec::new(),
        },
        // Here we could, for example, synchronize the model to an external
        // data store.
        |_model| (),
        // We need to use the `cx.build(...)` pattern similar to [`with`].
        |cx, model| cx.build(tutorial(model)),
    );
}
