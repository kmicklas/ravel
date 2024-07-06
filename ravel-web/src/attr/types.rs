//! HTML attribute types.
//!
//! Usually you shouldn't need to import or reference these directly.

use std::marker::PhantomData;

use ravel::{Builder, State};
use wasm_bindgen::UnwrapThrowExt;

use crate::{BuildCx, RebuildCx, Web};

use super::CloneString;

/// Trait to identify attribute types.
pub trait AttrKind: 'static {
    /// The name of the attribute.
    const NAME: &'static str;
}

pub trait AttrValue {
    type Saved: 'static;

    fn save(self) -> Self::Saved;

    fn changed(&self, saved: &Self::Saved) -> bool;

    fn with_str<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&str>) -> R;
}

impl<V: AttrValue> AttrValue for Option<V> {
    type Saved = Option<V::Saved>;

    fn save(self) -> Self::Saved {
        self.map(AttrValue::save)
    }

    fn changed(&self, saved: &Self::Saved) -> bool {
        match (self, saved) {
            (None, None) => false,
            (None, Some(_)) => true,
            (Some(_), None) => true,
            (Some(v), Some(saved)) => v.changed(saved),
        }
    }

    fn with_str<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&str>) -> R,
    {
        match self {
            Some(v) => v.with_str(f),
            None => f(None),
        }
    }
}

impl<V: AsRef<str>> AttrValue for CloneString<V> {
    type Saved = String;

    fn save(self) -> Self::Saved {
        self.0.as_ref().to_string()
    }

    fn changed(&self, saved: &Self::Saved) -> bool {
        self.0.as_ref() != saved.as_str()
    }

    fn with_str<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&str>) -> R,
    {
        f(Some(self.0.as_ref()))
    }
}

impl AttrValue for &'static str {
    type Saved = Self;

    fn save(self) -> Self::Saved {
        self
    }

    fn changed(&self, saved: &Self::Saved) -> bool {
        !std::ptr::eq(*self, *saved)
    }

    fn with_str<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&str>) -> R,
    {
        f(Some(self))
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct BooleanAttrValue(pub bool);

impl AttrValue for BooleanAttrValue {
    type Saved = bool;

    fn save(self) -> Self::Saved {
        self.0
    }

    fn changed(&self, saved: &Self::Saved) -> bool {
        self.0 != *saved
    }

    fn with_str<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&str>) -> R,
    {
        f(if self.0 { Some("") } else { None })
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
pub trait ClassValue: 'static + PartialEq {
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

#[doc(hidden)]
pub struct Classes<V: ClassValue>(pub V);

impl<V: ClassValue> AttrValue for Classes<V> {
    type Saved = V; // TODO: Associated saved type

    fn save(self) -> Self::Saved {
        self.0
    }

    fn changed(&self, saved: &Self::Saved) -> bool {
        self.0 != *saved
    }

    fn with_str<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&str>) -> R,
    {
        match self.0.as_str() {
            Some(s) => f(Some(s)),
            None => {
                let mut s = String::new();

                self.0.for_each(|c| {
                    if !s.is_empty() {
                        s.push(' ');
                    }

                    s.push_str(c);
                });

                f(if s.is_empty() { None } else { Some(&s) })
            }
        }
    }
}

/// The state of an [`Attr`].
pub struct AttrState<Saved> {
    value: Saved,
}

impl<Saved> AttrState<Saved> {
    fn build<V: AttrValue<Saved = Saved>>(
        parent: &web_sys::Element,
        name: &'static str,
        value: V,
    ) -> Self {
        value.with_str(|value| {
            if let Some(value) = value {
                parent.set_attribute(name, value).unwrap_throw()
            }
        });

        Self {
            value: value.save(),
        }
    }

    fn rebuild<V: AttrValue<Saved = Saved>>(
        &mut self,
        parent: &web_sys::Element,
        name: &'static str,
        value: V,
    ) {
        if !value.changed(&self.value) {
            return;
        }

        value.with_str(|value| {
            if let Some(value) = value {
                parent.set_attribute(name, value).unwrap_throw()
            } else {
                parent.remove_attribute(name).unwrap_throw()
            }
        });
    }
}

impl<Saved: 'static, Output> State<Output> for AttrState<Saved> {
    fn run(&mut self, _: &mut Output) {}
}

/// An arbitrary attribute.
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Attr<Kind: AttrKind, Value> {
    pub(crate) value: Value,
    pub(crate) kind: PhantomData<Kind>,
}

impl<Kind: AttrKind, Value: AttrValue> Builder<Web> for Attr<Kind, Value> {
    type State = AttrState<Value::Saved>;

    fn build(self, cx: BuildCx) -> Self::State {
        AttrState::build(cx.position.parent, Kind::NAME, self.value)
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        state.rebuild(cx.parent, Kind::NAME, self.value)
    }
}

macro_rules! make_attr_value_type {
    ($name:literal, $t:ident, $value_type:ty) => {
        make_attr_value_type_state!(
            $name,
            $t,
            $value_type,
            std::convert::identity,
            <V as AttrValue>::Saved
        );
    };
    ($name:literal, $t:ident, $value_type:ty, $value_wrapper:ident) => {
        make_attr_value_type_state!(
            $name,
            $t,
            $value_type,
            $value_wrapper,
            <$value_wrapper as AttrValue>::Saved
        );
    };
}

macro_rules! make_attr_value_type_state {
    ($name:literal, $t:ident, $value_type:ty, $value_wrapper:expr, $state_value:ty) => {
        #[doc = concat!("`", $name, "` attribute.")]
        #[derive(Copy, Clone)]
        pub struct $t(pub $value_type);

        impl Builder<Web> for $t {
            type State = AttrState<$state_value>;

            fn build(self, cx: BuildCx) -> Self::State {
                AttrState::build(
                    cx.position.parent,
                    $name,
                    $value_wrapper(self.0),
                )
            }

            fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
                state.rebuild(cx.parent, $name, $value_wrapper(self.0))
            }
        }
    };
}

macro_rules! make_attr_value_trait {
    ($name:literal, $t:ident, $value_trait:ident) => {
        make_attr_value_trait_state!(
            $name,
            $t,
            $value_trait,
            std::convert::identity,
            <V as AttrValue>::Saved
        );
    };
    ($name:literal, $t:ident, $value_trait:ident, $value_wrapper:ident) => {
        make_attr_value_trait_state!(
            $name,
            $t,
            $value_trait,
            $value_wrapper,
            <$value_wrapper<V> as AttrValue>::Saved
        );
    };
}

macro_rules! make_attr_value_trait_state {
    ($name:literal, $t:ident, $value_trait:ident, $value_wrapper:expr, $state_value:ty) => {
        #[doc = concat!("`", $name, "` attribute.")]
        #[derive(Copy, Clone)]
        pub struct $t<V: $value_trait>(pub V);

        impl<V: $value_trait> Builder<Web> for $t<V> {
            type State = AttrState<$state_value>;

            fn build(self, cx: BuildCx) -> Self::State {
                AttrState::build(
                    cx.position.parent,
                    $name,
                    $value_wrapper(self.0),
                )
            }

            fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
                state.rebuild(cx.parent, $name, $value_wrapper(self.0))
            }
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/gen_attr_types.rs"));
