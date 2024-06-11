use std::marker::PhantomData;

use crate::{with, Builder, Cx, CxRep, State, Token};

/// A [`Builder`] created from [`with_local`].
pub struct WithLocal<Init, F, S> {
    init: Init,
    f: F,
    phantom: PhantomData<S>,
}

impl<R: CxRep, T, Init, F, S> Builder<R> for WithLocal<Init, F, S>
where
    Init: FnOnce() -> T,
    F: FnOnce(Cx<S, R>, &T) -> Token<S>,
{
    type State = WithLocalState<T, S>;

    fn build(self, cx: R::BuildCx<'_>) -> Self::State {
        let value = (self.init)();
        let inner = with(|cx| (self.f)(cx, &value)).build(cx);
        WithLocalState { value, inner }
    }

    fn rebuild(self, cx: R::RebuildCx<'_>, state: &mut Self::State) {
        with(|cx| (self.f)(cx, &mut state.value)).rebuild(cx, &mut state.inner)
    }
}

/// The state of a [`WithLocal`].
pub struct WithLocalState<T, S> {
    value: T,
    inner: S,
}

impl<Output: Default, T: 'static + Default, S> State<Output>
    for WithLocalState<T, S>
where
    S: State<(Output, T)>,
{
    fn run(&mut self, output: &mut Output) {
        let mut data =
            (std::mem::take(output), std::mem::take(&mut self.value));
        self.inner.run(&mut data);
        (*output, self.value) = data;
    }
}

/// Creates a [`Builder`] which has access to a local state value.
///
/// The `init` callback determines the inital value of the local state, and will
/// only be run when the component is initially built.
///
/// Like [`with`], `f` must call [`Cx::build`] to return a [`Token`].
pub fn with_local<T, Init, F, S, R: CxRep>(
    init: Init,
    f: F,
) -> WithLocal<Init, F, S>
where
    Init: FnOnce() -> T,
    F: FnOnce(Cx<S, R>, &T) -> Token<S>,
{
    WithLocal {
        init,
        f,
        phantom: PhantomData,
    }
}
