//! HTML attributes.

use std::marker::PhantomData;

use ravel::Builder;

use crate::{BuildCx, RebuildCx, Web};

use self::types::*;

pub mod types;

// TODO: Dedup with `Text`/`text`? It's the same thing for text nodes.
/// A string type which is cloned to [`String`] to use as an attribute value.
///
/// This wrapepr type exists to draw attention to the fact that using a borrowed
/// string value requires cloning to a persistent string in the attribute state.
/// This also permits a more efficient implementation of `&'static str`, which
/// does not need this wrapper.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CloneString<V: AsRef<str>>(pub V);

/// An arbitrary attribute.
pub fn attr<Kind: AttrKind, Value: AttrValue>(
    _: Kind,
    value: Value,
) -> Attr<Kind, Value> {
    Attr {
        value,
        kind: PhantomData,
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

include!(concat!(env!("OUT_DIR"), "/gen_attr.rs"));
