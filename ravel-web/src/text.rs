//! Text nodes and formatting.

use std::{
    borrow::Cow,
    fmt::{Arguments, Write},
};

use ravel::Builder;
use web_sys::wasm_bindgen::UnwrapThrowExt;

use crate::{BuildCx, RebuildCx, State, View, Web};

/// A text node.
pub struct Text<Value: ToString + AsRef<str>> {
    value: Value,
}

impl<Value: ToString + AsRef<str>> View for Text<Value> {}
impl<Value: ToString + AsRef<str>> Builder<Web> for Text<Value> {
    type State = TextState<String>;

    fn build(self, cx: BuildCx) -> Self::State {
        let node =
            web_sys::Text::new_with_data(self.value.as_ref()).unwrap_throw();

        cx.position.insert(&node);

        TextState {
            node,
            value: self.value.to_string(),
        }
    }

    fn rebuild(self, _: RebuildCx, state: &mut Self::State) {
        if state.value != self.value.as_ref() {
            state.node.set_data(self.value.as_ref());
            state.value = self.value.to_string();
        }
    }
}

/// The state of a [`Text`].
pub struct TextState<Value> {
    node: web_sys::Text,
    value: Value,
}

impl<Output, Value: 'static> State<Output> for TextState<Value> {
    fn run(&mut self, _: &mut Output) {}
}

/// A text node.
pub fn text<V: ToString + AsRef<str>>(value: V) -> Text<V> {
    Text { value }
}

impl View for &'static str {}
impl Builder<Web> for &'static str {
    type State = TextState<Self>;

    fn build(self, cx: BuildCx) -> Self::State {
        let node = web_sys::Text::new_with_data(self).unwrap_throw();

        cx.position.insert(&node);

        TextState { node, value: self }
    }

    fn rebuild(self, _: RebuildCx, state: &mut Self::State) {
        if !std::ptr::eq(self, state.value) {
            state.node.set_data(self);
            state.value = self;
        }
    }
}

impl<'a> View for Arguments<'a> {}
impl<'a> Builder<Web> for Arguments<'a> {
    type State = TextState<Cow<'static, str>>;

    fn build(self, cx: BuildCx) -> Self::State {
        let value = match self.as_str() {
            Some(s) => Cow::Borrowed(s),
            None => Cow::Owned(self.to_string()),
        };

        let node = web_sys::Text::new_with_data(&value).unwrap_throw();

        cx.position.insert(&node);

        TextState { node, value }
    }

    fn rebuild(self, _: RebuildCx, state: &mut Self::State) {
        match self.as_str() {
            Some(new) => {
                if !match &state.value {
                    Cow::Borrowed(old) => std::ptr::eq(new, *old),
                    Cow::Owned(old) => new == old,
                } {
                    state.node.set_data(new);
                    state.value = Cow::Borrowed(new);
                }
            }
            None => match &mut state.value {
                Cow::Borrowed(_) => {
                    let new = self.to_string();
                    state.node.set_data(&new);
                    state.value = Cow::Owned(new);
                }
                Cow::Owned(value) => {
                    let mut w = UpdateString {
                        value,
                        index: 0,
                        changed: false,
                    };

                    std::fmt::write(&mut w, self).unwrap_throw();

                    if w.changed {
                        state.node.set_data(value);
                    }
                }
            },
        }
    }
}

struct UpdateString<'a> {
    value: &'a mut String,
    index: usize,
    changed: bool,
}

impl<'a> Write for UpdateString<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let remaining = &self.value[self.index..];

        if remaining.strip_prefix(s).is_none() {
            self.value.truncate(self.index);
            self.value.push_str(s);
            self.changed = true;
        }
        self.index += s.len();

        Ok(())
    }
}

#[doc(hidden)]
pub mod reexport {
    pub use ravel::with;
}

/// Displays text with a format string.
///
/// Once [rust-lang/rust#92698](https://github.com/rust-lang/rust/issues/92698)
/// is fixed, it will be possible to use [`format_args`] directly.
#[macro_export]
macro_rules! format_text {
    ($fmt:expr) => {
        $crate::text::reexport::with(move |cx| cx.build(::std::format_args!($fmt)))
    };
    ($fmt:expr, $($args:tt)*) => {
        $crate::text::reexport::with(move |cx| cx.build(::std::format_args!($fmt, $($args)*)))
    };
}
