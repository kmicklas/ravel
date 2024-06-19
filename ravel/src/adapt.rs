use std::marker::PhantomData;

use crate::{Builder, CxRep, State};

/// A [`Builder`] created from [`adapt`].
pub struct Adapt<B, F, S, Output> {
    builder: B,
    f: F,
    phantom: PhantomData<(S, Output)>,
}

impl<R: CxRep, B, F, S, Output> Builder<R> for Adapt<B, F, S, Output>
where
    B: Builder<R>,
{
    type State = AdaptState<B::State, F>;

    fn build(self, cx: R::BuildCx<'_>) -> Self::State {
        AdaptState {
            inner: self.builder.build(cx),
            f: self.f,
        }
    }

    fn rebuild(self, cx: R::RebuildCx<'_>, state: &mut Self::State) {
        state.f = self.f;
        self.builder.rebuild(cx, &mut state.inner)
    }
}

/// A reference to a [`State`] which must be [`State::run`].
pub struct Thunk<'s, S> {
    state: &'s mut S,
}

/// The result of [`Thunk::run`].
pub struct ThunkResult<S> {
    phantom: PhantomData<S>,
}

impl<'s, S> Thunk<'s, S> {
    /// Consumes the [`Thunk`], invoking [`State::run`] on `S` with `output`.
    pub fn run<Output>(self, output: &mut Output) -> ThunkResult<S>
    where
        S: State<Output>,
    {
        self.state.run(output);
        ThunkResult {
            phantom: PhantomData,
        }
    }
}

/// The state of an [`Adapt`].
pub struct AdaptState<S, F> {
    inner: S,
    f: F,
}

impl<S: 'static, F, Output> State<Output> for AdaptState<S, F>
where
    F: 'static + FnMut(Thunk<S>, &mut Output) -> ThunkResult<S>,
{
    fn run(&mut self, output: &mut Output) {
        (self.f)(
            Thunk {
                state: &mut self.inner,
            },
            output,
        );
    }
}

/// Adapts a [`Builder`] so that its [`State`] is compatible with a different
/// `Output` type.
///
/// The provided callback must call [`Thunk::run`] with an adapted reference.
pub fn adapt<B, F, S, Output>(builder: B, f: F) -> Adapt<B, F, S, Output>
where
    F: 'static + FnMut(Thunk<S>, &mut Output) -> ThunkResult<S>,
{
    Adapt {
        builder,
        f,
        phantom: PhantomData,
    }
}

/// Adapts a [`Builder`] so that its [`State`] is compatible with a different
/// `Output` type.
///
/// This is a simpler variant of [`adapt`] for the common case where the adapted
/// reference can be borrowed from `Output`.
pub fn adapt_ref<B, F, S, Output, R>(
    builder: B,
    mut f: F,
) -> Adapt<
    B,
    // TODO: Remove this impl trait
    impl 'static + FnMut(Thunk<S>, &mut Output) -> ThunkResult<S>,
    S,
    Output,
>
where
    F: 'static + FnMut(&mut Output) -> &mut R,
    S: State<R>,
{
    adapt(builder, move |thunk, output| thunk.run(f(output)))
}
