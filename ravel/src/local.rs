use std::marker::PhantomData;

use crate::{with, Builder, Cx, CxRep, Float, State, Token};

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
        WithLocalState {
            value: Float::new(value),
            inner,
        }
    }

    fn rebuild(self, cx: R::RebuildCx<'_>, state: &mut Self::State) {
        with(|cx| (self.f)(cx, state.value.as_ref().unwrap()))
            .rebuild(cx, &mut state.inner)
    }
}

/// The state of a [`WithLocal`].
pub struct WithLocalState<T, S> {
    pub value: Float<T>,
    pub inner: S,
}

impl<Output, T: 'static, S> State<Output> for WithLocalState<T, S>
where
    S: State<(Output, T)>,
{
    fn run(&mut self, output: &mut Float<Output>) {
        output
            .float_(|output| {
                self.value
                    .float(|value| {
                        let mut data = Float::new((output, value));
                        self.inner.run(&mut data);
                        let (output, value) = data.into_inner().unwrap();
                        (value, output)
                    })
                    .unwrap()
            })
            .unwrap()
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
