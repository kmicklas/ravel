//! HTML attributes.

use std::marker::PhantomData;

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

include!(concat!(env!("OUT_DIR"), "/gen_attr.rs"));
