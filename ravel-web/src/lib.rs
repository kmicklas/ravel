//! A web/DOM/HTML backend for [`ravel`].

#![feature(trait_upcasting)]
use std::{any::Any, sync::Arc};

use atomic_waker::AtomicWaker;
use dom::Position;
use ravel::{Builder, Cx, CxRep};

mod any;
pub mod attr;
mod dom;
pub mod el;
pub mod event;
pub mod list;
mod option;
pub mod run;
pub mod text;

pub use any::*;
pub use option::*;

/// A dummy type representing the web backend.
pub struct Web;

impl CxRep for Web {
    type BuildCx<'a> = BuildCx<'a>;
    type RebuildCx<'a> = RebuildCx<'a>;
}

/// The necessary context for building [`Web`] components.
#[derive(Copy, Clone)]
pub struct BuildCx<'cx> {
    position: Position<'cx>,
}

/// The necessary context for rebuilding [`Web`] components.
#[derive(Copy, Clone)]
pub struct RebuildCx<'cx> {
    parent: &'cx web_sys::Element,
    // TODO: Remove double pointer.
    waker: &'cx Arc<AtomicWaker>,
}

/// Trait for the state of a [`Web`] component.
pub trait State<Output>: Any {
    /// Processes a "frame".
    ///
    /// This method can respond to externally triggered events by changing the
    /// `Output`.
    fn run(&mut self, output: &mut Output);
}

macro_rules! tuple_state {
    ($($a:ident),*) => {
        #[allow(non_camel_case_types)]
        impl<$($a,)* O> State<O> for ($($a,)*)
        where
            $($a: State<O>,)*
        {
            fn run(&mut self, _output: &mut O) {
                let ($($a,)*) = self;
                $($a.run(_output);)*
            }
        }
    };
}

tuple_state!();
tuple_state!(a);
tuple_state!(a, b);
tuple_state!(a, b, c);
tuple_state!(a, b, c, d);
tuple_state!(a, b, c, d, e);
tuple_state!(a, b, c, d, e, f);
tuple_state!(a, b, c, d, e, f, g);
tuple_state!(a, b, c, d, e, f, g, h);

/// Trait for DOM fragments.
///
/// These types can be used in contexts where the component may be removed
/// according to its position in the DOM, for example when used inside an
/// [`Option`].
///
/// This is implemented for [`el`] and [`text`] types and composites thereof,
/// but not for [`attr`] or [`event`] types, which must always be permanently
/// attached to an element.
pub trait View: Builder<Web> {}

macro_rules! tuple_view {
    ($($a:ident),*) => {
        #[allow(non_camel_case_types)]
        impl<$($a: View),*> View for ($($a,)*) {}
    };
}

tuple_view!();
tuple_view!(a);
tuple_view!(a, b);
tuple_view!(a, b, c);
tuple_view!(a, b, c, d);
tuple_view!(a, b, c, d, e);
tuple_view!(a, b, c, d, e, f);
tuple_view!(a, b, c, d, e, f, g);
tuple_view!(a, b, c, d, e, f, g, h);
