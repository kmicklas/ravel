//! HTML attributes.

use std::marker::PhantomData;

use ravel::{Builder, Float, State};
use web_sys::wasm_bindgen::UnwrapThrowExt as _;

use crate::{BuildCx, RebuildCx, Web};

/// Trait to identify attribute types.
pub trait AttrKind: 'static {
    /// The name of the attribute.
    const NAME: &'static str;
}

/// An arbitrary attribute.
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Attr<Kind: AttrKind, Value> {
    value: Value,
    kind: PhantomData<Kind>,
}

impl<Kind: AttrKind, Value: AsRef<str>> Builder<Web> for Attr<Kind, Value> {
    type State = AttrState;

    fn build(self, cx: BuildCx) -> Self::State {
        AttrState::build(cx, Kind::NAME, self.value.as_ref())
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        state.rebuild(cx.parent, Kind::NAME, self.value.as_ref())
    }
}

/// The state of an [`Attr`].
pub struct AttrState {
    value: String,
}

impl AttrState {
    fn build(cx: BuildCx, name: &'static str, value: &str) -> Self {
        cx.position.parent.set_attribute(name, value).unwrap_throw();

        Self {
            value: value.to_string(),
        }
    }

    fn rebuild(
        &mut self,
        parent: &web_sys::Element,
        name: &'static str,
        value: &str,
    ) {
        if self.value == value {
            return;
        }

        self.value = value.to_string();
        parent.set_attribute(name, value).unwrap_throw()
    }
}

impl<Output> State<Output> for AttrState {
    fn run(&mut self, _: &mut Float<Output>) {}
}

/// An arbitrary attribute.
pub fn attr<Kind: AttrKind, Value>(_: Kind, value: Value) -> Attr<Kind, Value> {
    Attr {
        value,
        kind: PhantomData,
    }
}

/// Trait for `class` attribute values.
///
/// In HTML, `class` is a space separated list. Rather than requiring you to
/// construct the string by hand, this trait allows easily constructing it from
/// various types:
///
/// * A [`String`] or `&'static str` is just a class name.
/// * A tuple of `ClassValue`s is the union of the component class names.
/// * An [`Option<T>`] is an optional set of classes.
pub trait ClassValue: Eq {
    /// If the value is available as a static string, providing it allows a more
    /// efficient implementation. The default implementation returns [`None`].
    fn as_str(&self) -> Option<&'static str> {
        None
    }

    /// Calls a callback for each class name.
    fn for_each<F: FnMut(&str)>(&self, f: F);
}

impl ClassValue for &'static str {
    fn as_str(&self) -> Option<&'static str> {
        Some(self)
    }

    fn for_each<F: FnMut(&str)>(&self, mut f: F) {
        f(self)
    }
}

impl ClassValue for String {
    fn for_each<F: FnMut(&str)>(&self, mut f: F) {
        f(self)
    }
}

impl<C: ClassValue> ClassValue for Option<C> {
    fn as_str(&self) -> Option<&'static str> {
        self.as_ref().and_then(C::as_str)
    }

    fn for_each<F: FnMut(&str)>(&self, f: F) {
        if let Some(s) = self.as_ref() {
            s.for_each(f);
        }
    }
}

macro_rules! tuple_class_value {
    ($($a:ident),*) => {
        #[allow(non_camel_case_types)]
        impl<$($a: ClassValue),*> ClassValue for ($($a,)*) {
            fn for_each<F: FnMut(&str)>(&self, mut _f: F) {
                let ($($a,)*) = self;
                $($a.for_each(&mut _f);)*
            }
        }
    };
}

tuple_class_value!();
tuple_class_value!(a);
tuple_class_value!(a, b);
tuple_class_value!(a, b, c);
tuple_class_value!(a, b, c, d);
tuple_class_value!(a, b, c, d, e);
tuple_class_value!(a, b, c, d, e, f);
tuple_class_value!(a, b, c, d, e, f, g);
tuple_class_value!(a, b, c, d, e, f, g, h);

/// `class` attribute.
pub struct AttrClass<Value> {
    value: Value,
}

impl<Value: ClassValue> AttrClass<Value> {
    fn set_on(self, parent: &web_sys::Element) -> Value {
        let mut s = String::new();

        parent
            .set_attribute(
                "class",
                match self.value.as_str() {
                    Some(s) => s,
                    None => {
                        self.value.for_each(|c| {
                            if !s.is_empty() {
                                s.push(' ');
                            }

                            s.push_str(c);
                        });
                        &s
                    }
                },
            )
            .unwrap_throw();

        self.value
    }
}

impl<Value: ClassValue> Builder<Web> for AttrClass<Value> {
    type State = AttrClassState<Value>;

    fn build(self, cx: BuildCx) -> Self::State {
        AttrClassState {
            value: self.set_on(cx.position.parent),
        }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        if state.value != self.value {
            state.value = self.set_on(cx.parent);
        }
    }
}

/// The state of an [`AttrClass`].
pub struct AttrClassState<Value> {
    value: Value,
}

impl<Value: 'static, Output> State<Output> for AttrClassState<Value> {
    fn run(&mut self, _: &mut Float<Output>) {}
}

/// `class` attribute.
pub fn class<Value: ClassValue>(value: Value) -> AttrClass<Value> {
    AttrClass { value }
}

/// An arbitrary boolean attribute.
pub struct BooleanAttr<Kind: AttrKind> {
    value: bool,
    kind: PhantomData<Kind>,
}

impl<Kind: AttrKind> Builder<Web> for BooleanAttr<Kind> {
    type State = BooleanAttrState;

    fn build(self, cx: BuildCx) -> Self::State {
        BooleanAttrState::build(cx, Kind::NAME, self.value)
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        state.rebuild(cx.parent, Kind::NAME, self.value)
    }
}

/// The state of a [`BooleanAttr`].
pub struct BooleanAttrState {
    value: bool,
}

impl BooleanAttrState {
    fn build(cx: BuildCx, name: &'static str, value: bool) -> Self {
        if value {
            cx.position.parent.set_attribute(name, "").unwrap_throw()
        }

        Self { value }
    }

    fn rebuild(
        &mut self,
        parent: &web_sys::Element,
        name: &'static str,
        value: bool,
    ) {
        if value && !self.value {
            parent.set_attribute(name, "").unwrap_throw()
        } else if !value && self.value {
            parent.remove_attribute(name).unwrap_throw()
        }
        self.value = value;
    }
}

impl<Output> State<Output> for BooleanAttrState {
    fn run(&mut self, _: &mut Float<Output>) {}
}

/// An arbitrary boolean attribute.
pub fn boolean_attr<Kind: AttrKind>(_: Kind, value: bool) -> BooleanAttr<Kind> {
    BooleanAttr {
        value,
        kind: PhantomData,
    }
}

macro_rules! attr_kind {
    ($t:ident, $name:expr) => {
        #[doc = concat!("`", $name, "` attribute.")]
        #[derive(Copy, Clone)]
        pub struct $t;

        impl AttrKind for $t {
            const NAME: &'static str = $name;
        }
    };
}

macro_rules! make_attr {
    ($name:ident, $t:ident) => {
        make_attr!(stringify!($name), $name, $t);
    };
    ($name:expr, $f:ident, $t:ident) => {
        attr_kind!($t, $name);

        #[doc = concat!("`", $name, "` attribute.")]
        pub fn $f<Value>(value: Value) -> Attr<$t, Value> {
            attr($t, value)
        }
    };
}

make_attr!("aria-hidden", aria_hidden, AriaHidden); // TODO: typed
make_attr!("for", for_, For);
make_attr!(href, Href);
make_attr!(id, Id);
make_attr!(max, Max);
make_attr!(min, Min);
make_attr!("value", value_, Value_);
make_attr!(placeholder, Placeholder);
make_attr!(style, Style);
make_attr!("type", type_, Type);

macro_rules! make_boolean_attr {
    ($name:ident, $t:ident) => {
        make_boolean_attr!(stringify!($name), $name, $t);
    };
    ($name:expr, $f:ident, $t:ident) => {
        #[doc = concat!("`", $name, "` attribute.")]
        #[repr(transparent)]
        #[derive(Copy, Clone, Debug)]
        pub struct $t(bool);

        impl Builder<Web> for $t {
            type State = BooleanAttrState;

            fn build(self, cx: BuildCx) -> Self::State {
                BooleanAttrState::build(cx, $name, self.0)
            }

            fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
                state.rebuild(cx.parent, $name, self.0)
            }
        }

        #[doc = concat!("`", $name, "` attribute.")]
        pub fn $f(value: bool) -> $t {
            $t(value)
        }
    };
}

make_boolean_attr!(autofocus, Autofocus);
make_boolean_attr!(checked, Checked);
