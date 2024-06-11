use std::collections::BTreeMap;

use ravel::{with, with_local};
use ravel_web::{
    any, attr,
    collections::{btree_map, slice},
    el,
    event::{self, on, on_},
    format_text,
    run::spawn_body,
    text::{display, text},
    View,
};
use web_sys::{
    wasm_bindgen::{JsCast as _, UnwrapThrowExt},
    HtmlInputElement,
};

/// Our model type contains the global state of the application.
#[derive(Default)]
struct Model {
    count: usize,
    message: String,
    item_map: BTreeMap<usize, String>,
    item_vec: Vec<(usize, String)>,
}

/// We can build our application modularly out of components. A component is
/// just some function which returns a [`View!`].
///
/// ([`View!`] is just a convenience macro for a slightly verbose constraint on
/// [`trait@View`].)
fn basic_html() -> View!(Model) {
    // We can build views out of HTML elements, which are defined in the
    // [`el`] module. These take another view parameter for their body.
    //
    // We can also compose views using tuples.
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
/// appropriate bound to the return type for the captured lifetime (here `'_`).
fn state(model: &Model) -> View!(Model, '_) {
    (
        el::h2("State"),
        el::p(
            // To generate strings dynamically, we can use standard format
            // strings using [`format_text`].
            format_text!("Count: {}", model.count),
        ),
        el::p((
            "Also count: ",
            // In the very common case of just displaying a scalar value like a
            // number, it is easier and more efficient to use [`display`].
            display(model.count),
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
/// [`Model`] parameter to [`View!`]. Event handlers have mutable access to this
/// type when they run.
fn events() -> View!(Model) {
    (
        el::h2("Events"),
        el::p(el::button((
            "Increment count",
            // We can update the model in response to a chosen HTML event type.
            on_(event::Click, |model: &mut Model| {
                model.item_map.insert(model.count, model.message.clone());
                model.item_vec.push((model.count, model.message.clone()));
                model.count += 1;
            }),
        ))),
        el::p((
            "Message: ",
            // [`on`], unlike [`on_`], also gives us access to the underlying
            // [`web_sys::Event`].
            el::input(on(event::InputEvent, |model: &mut Model, event| {
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

/// Sometimes, we might not want to store all state in our global [`Model`].
///
/// This is useful when it would be tedious to write down uninteresting state
/// types, or when you want to encapsulate the behavior of a reusable component.
/// However, it generally increases complexity and makes testing harder.
fn local_state() -> View!(Model) {
    with_local(
        // We provide an initialization callback, which is only run when the
        // component is constructed for the first time.
        || 0,
        |cx, local_count| {
            // Inside the body, we have a reference to the current local state.
            cx.build((
                el::h2("Local state"),
                el::p(("Local count: ", display(*local_count))),
                el::p(el::button((
                    "Increment local count",
                    // Although we have a reference to the current value, we
                    // cannot mutate it, or store it in an event handler (which
                    // must remain `'static`).
                    //
                    // Instead, [`with_local`] changes our state type to be a
                    // tuple which has both the outer state ([`Model`]) and our
                    // local state type.
                    on_(event::Click, move |(_model, local_count): &mut _| {
                        *local_count += 1;
                    }),
                ))),
            ))
        },
    )
}

/// All of our views so far have had a static structure. Sometimes, we need to
/// swap out or hide various components.
fn dynamic_view(model: &Model) -> View!(Model, '_) {
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
/// [`trait@View`] over a [`BTreeMap`] with [`btree_map()`]. This is useful when
///  the entries are ordered by some type of key, but may be inserted or removed
/// at any position.
///
/// If the data is just an array which grows or shrinks at the end, we can use
/// [`slice()`] to generate a [`trait@View`] over a [`Vec`]/slice.
fn lists(model: &Model) -> View!(Model, '_) {
    (
        el::h2("Map view"),
        el::p(el::table((
            el::thead(el::tr((el::td(()), el::td("Id"), el::td("Message")))),
            el::tbody(btree_map(&model.item_map, |cx, key, value| {
                let key = *key;
                cx.build(el::tr((
                    el::td(el::button((
                        "Remove",
                        on_(event::Click, {
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
                        on_(event::Click, {
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
fn tutorial(model: &Model) -> View!(Model, '_) {
    (
        basic_html(),
        state(model),
        events(),
        local_state(),
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
