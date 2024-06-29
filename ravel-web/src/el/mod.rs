//! HTML elements.

use std::marker::PhantomData;

use self::types::{El, ElKind};

pub mod types;

/// An arbitrary element.
pub fn el<Kind: ElKind, Body>(_: Kind, body: Body) -> El<Kind, Body> {
    El {
        kind: PhantomData,
        body,
    }
}

include!(concat!(env!("OUT_DIR"), "/gen_el.rs"));
