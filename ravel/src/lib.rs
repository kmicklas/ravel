//! An experimental approach to UI in Rust with a focus on ergonomics,
//! efficiency, and simplicity.
//!
//! This crate contains shared infrastructure which is platform agnostic. To
//! build an application, you'll need a backend crate such as `ravel-web`.
use std::{marker::PhantomData, mem::MaybeUninit};

use paste::paste;

mod adapt;
mod any;
mod local;

pub use adapt::*;
pub use any::*;
pub use local::*;

/// A dummy type which typically represents a "backend".
pub trait CxRep {
    type BuildCx<'a>: Copy;
    type RebuildCx<'a>: Copy;
}

/// Initializes a component or applies updates to an existing one.
pub trait Builder<R: CxRep> {
    /// The associated state of the component.
    type State;

    fn build(self, cx: R::BuildCx<'_>) -> Self::State;
    fn rebuild(self, cx: R::RebuildCx<'_>, state: &mut Self::State);
}

macro_rules! tuple_builder {
    ($($a:ident),*) => {
        #[allow(non_camel_case_types)]
        impl<R: CxRep, $($a: Builder<R>,)*> Builder<R> for ($($a,)*) {
            type State = ($($a::State,)*);

            fn build(self, _cx: R::BuildCx<'_>) -> Self::State {
                let ($($a,)*) = self;
                #[allow(clippy::unused_unit)]
                ($($a.build(_cx),)*)
            }

            fn rebuild(self, _cx: R::RebuildCx<'_>, state: &mut Self::State) {
                let ($($a,)*) = self;
                let ($(paste!([< state_ $a >]),)*) = state;

                $($a.rebuild(_cx, paste!([< state_ $a >]));)*
            }
        }
    };
}

tuple_builder!();
tuple_builder!(a);
tuple_builder!(a, b);
tuple_builder!(a, b, c);
tuple_builder!(a, b, c, d);
tuple_builder!(a, b, c, d, e);
tuple_builder!(a, b, c, d, e, f);
tuple_builder!(a, b, c, d, e, f, g);
tuple_builder!(a, b, c, d, e, f, g, h);

/// Trait for the state of a [`Builder`].
pub trait State<Output>: AsAny {
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

/// Context provided by [`with`].
pub struct Cx<'cx, 'state, State, R: CxRep> {
    inner: CxInner<'cx, 'state, State, R>,
}

enum CxInner<'cx, 'state, State, R: CxRep> {
    Build {
        state: &'state mut MaybeUninit<State>,
        cx: R::BuildCx<'cx>,
    },
    Rebuild {
        state: &'state mut State,
        cx: R::RebuildCx<'cx>,
    },
}

/// The result of calling [`Cx::build`].
/// This ensures correct usage of [`with`].
pub struct Token<State> {
    phantom: PhantomData<State>,
}

impl<'cx, 'state, State, R: CxRep> Cx<'cx, 'state, State, R> {
    /// Consumes a [`Builder`], returning a [`Token`] which completes the
    /// component construction.
    pub fn build<B: Builder<R, State = State>>(
        self,
        builder: B,
    ) -> Token<B::State> {
        match self.inner {
            CxInner::Build { state, cx } => {
                let s = builder.build(cx);
                state.write(s);
            }
            CxInner::Rebuild { state, cx } => builder.rebuild(cx, state),
        }

        Token {
            phantom: PhantomData,
        }
    }
}

/// A [`Builder`] created from [`with`].
pub struct With<F, State> {
    f: F,
    phantom: PhantomData<State>,
}

impl<F, State, R: CxRep> Builder<R> for With<F, State>
where
    F: FnOnce(Cx<State, R>) -> Token<State>,
{
    type State = State;

    fn build(self, cx: R::BuildCx<'_>) -> Self::State {
        let mut state = MaybeUninit::<State>::uninit();

        (self.f)(Cx {
            inner: CxInner::Build {
                state: &mut state,
                cx,
            },
        });

        unsafe { state.assume_init() }
    }

    fn rebuild(self, cx: R::RebuildCx<'_>, state: &mut Self::State) {
        (self.f)(Cx {
            inner: CxInner::Rebuild { state, cx },
        });
    }
}

/// Creates a [`Builder`] from a callback which uses [`Cx::build`]. The
/// [`Builder`] passed with [`Cx::build`] can borrow local data in the callback,
/// without that lifetime being captured in the result.
pub fn with<F, State, R: CxRep>(f: F) -> With<F, State>
where
    F: FnOnce(Cx<State, R>) -> Token<State>,
{
    With {
        f,
        phantom: PhantomData,
    }
}
