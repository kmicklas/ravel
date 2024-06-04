use std::collections::BTreeMap;

use ravel_web::{
    attr::*, collections::btree_map, el::*, event::*, format_text,
    run::spawn_body, text::text, State, View,
};
use web_sys::wasm_bindgen::{JsCast as _, UnwrapThrowExt};

#[derive(Default, Clone)]
struct Model {
    filter: Filter,
    items: BTreeMap<usize, Item>,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Default, Debug)]
enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

#[derive(Default, Clone)]
struct Item {
    text: String,
    checked: bool,
    editing: bool,
}

impl Model {
    fn count(&self) -> usize {
        self.items.values().filter(|i| !i.checked).count()
    }

    fn add(&mut self, text: String) {
        let id = match self.items.last_key_value() {
            Some((k, _)) => *k + 1,
            None => 0,
        };

        self.items.insert(
            id,
            Item {
                text,
                checked: false,
                editing: false,
            },
        );
    }
}

impl Filter {
    fn button(self, selected: Self) -> impl View<State = impl State<Model>> {
        li(a((
            format_text!("{:?}", self),
            class((selected == self).then_some("selected")),
            on_(Click, move |model: &mut Model| model.filter = self),
        )))
    }
}

fn item(
    filter: Filter,
    id: usize,
    item: &Item,
) -> impl '_ + View<State = impl State<Model>> {
    let show = match filter {
        Filter::All => true,
        Filter::Active => !item.checked,
        Filter::Completed => item.checked,
    };

    show.then(|| {
        li((
            class((
                item.checked.then_some("completed"),
                item.editing.then_some("editing"),
            )),
            div((
                class("view"),
                input((
                    type_("checkbox"),
                    class("toggle"),
                    // TODO: avoid circular dependency
                    checked(item.checked),
                    on(InputEvent, move |model: &mut Model, e| {
                        let input: web_sys::HtmlInputElement =
                            e.target().unwrap_throw().dyn_into().unwrap_throw();
                        model.items.get_mut(&id).unwrap_throw().checked =
                            input.checked();
                    }),
                )),
                label((
                    text(&item.text),
                    on_(DblClick, move |model: &mut Model| {
                        model.items.get_mut(&id).unwrap_throw().editing = true
                    }),
                )),
                button((
                    class("destroy"),
                    on_(Click, move |model: &mut Model| {
                        model.items.remove(&id);
                    }),
                )),
            )),
            form((
                input((class("edit"), value_(&item.text))),
                on(Active(Submit), move |model: &mut Model, e| {
                    e.prevent_default();

                    let form: web_sys::HtmlFormElement =
                        e.target().unwrap_throw().dyn_into().unwrap_throw();
                    let input: web_sys::HtmlInputElement = form
                        .get_with_index(0)
                        .unwrap_throw()
                        .dyn_into()
                        .unwrap_throw();

                    model.items.get_mut(&id).unwrap_throw().text =
                        input.value();
                    model.items.get_mut(&id).unwrap_throw().editing = false;
                }),
            )),
        ))
    })
}

fn todomvc(model: &Model) -> impl '_ + View<State = impl State<Model>> {
    (
        section((
            class("todoapp"),
            header((
                class("header"),
                h1("todos"),
                form((
                    input((
                        class("new-todo"),
                        placeholder("What needs to be done?"),
                        autofocus(true),
                    )),
                    on(Active(Submit), move |model: &mut Model, e| {
                        e.prevent_default();

                        let form: web_sys::HtmlFormElement =
                            e.target().unwrap_throw().dyn_into().unwrap_throw();
                        let input: web_sys::HtmlInputElement = form
                            .elements()
                            .get_with_index(0)
                            .unwrap_throw()
                            .dyn_into()
                            .unwrap_throw();

                        model.add(input.value());
                        input.set_value(""); // TOOD: clear input with framework
                    }),
                )),
            )),
            section((
                class("main"),
                input((
                    id("toggle-all"),
                    class("toggle-all"),
                    type_("checkbox"),
                )),
                label((for_("toggle-all"), "Mark all as complete")),
                ul((
                    class("todo-list"),
                    btree_map(&model.items, |cx, id, i| {
                        cx.build(item(model.filter, *id, i))
                    }),
                )),
            )),
            footer((
                class("footer"),
                span((
                    class("todo-count"),
                    strong(format_text!(
                        "{} {} left",
                        model.count(),
                        match model.count() {
                            1 => "item",
                            _ => "items",
                        }
                    )),
                )),
                ul((
                    class("filters"),
                    // TODO: array impls
                    Filter::All.button(model.filter),
                    Filter::Active.button(model.filter),
                    Filter::Completed.button(model.filter),
                )),
                button((
                    class("clear-completed"),
                    "Clear completed",
                    on_(Click, move |model: &mut Model| {
                        model.items.retain(|_, i| !i.checked)
                    }),
                )),
            )),
        )),
        footer((class("info"), p("Double-click to edit a todo"))),
    )
}

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Trace).unwrap();

    spawn_body(
        Default::default(),
        |_| (),
        |cx, model| cx.build(todomvc(model)),
    );
}
