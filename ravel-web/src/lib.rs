//! A web/DOM/HTML backend for [`ravel`].

use std::sync::Arc;

use atomic_waker::AtomicWaker;
use dom::Position;
use ravel::{AdaptState, Builder, Cx, CxRep, WithLocalState};

mod any;
pub mod attr;
pub mod collections;
mod dom;
pub mod el;
pub mod event;
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

/// A marker trait for the [`ravel::State`] types of a [`trait@View`].
pub trait ViewMarker {}

impl<T: 'static, S: ViewMarker> ViewMarker for WithLocalState<T, S> {}
impl<S: ViewMarker, F> ViewMarker for AdaptState<S, F> {}

macro_rules! tuple_state {
    ($($a:ident),*) => {
        #[allow(non_camel_case_types)]
        impl<$($a),*> ViewMarker for ($($a,)*)
        where
            $($a: ViewMarker,)*
        {
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
pub trait View: Builder<Web, State = Self::ViewState> {
    type ViewState: ViewMarker;
}

impl<T, S: ViewMarker> View for T
where
    T: Builder<Web, State = S>,
{
    type ViewState = S;
}

#[doc(hidden)]
pub use ravel::State as ViewState;

/// A convenience macro for declaring a [`trait@View`] type.
///
/// Takes as a parameter the `Output` type of the [`trait@View`]'s
/// [`Builder::State`].
#[macro_export]
macro_rules! View {
    ($output:ty) => {
        impl $crate::View<
            ViewState = impl use<> + $crate::ViewMarker + $crate::ViewState<$output>
        >
    };
}
