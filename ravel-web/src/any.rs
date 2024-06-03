use std::{any::Any, marker::PhantomData, ops::DerefMut};

use web_sys::wasm_bindgen::UnwrapThrowExt as _;

use crate::{
    dom::{clear, Position},
    BuildCx, Builder, RebuildCx, State, View, Web,
};

/// A wrapper around a [`View`], erasing its [`State`] type.
pub struct AnyView<V: View, Output> {
    inner: V,
    phantom: PhantomData<fn(&mut Output)>,
}

impl<V: View, Output: 'static> View for AnyView<V, Output> where
    V::State: State<Output>
{
}
impl<V: View, Output: 'static> Builder<Web> for AnyView<V, Output>
where
    V::State: State<Output>,
{
    type State = AnyState<Output>;

    fn build(self, cx: BuildCx) -> Self::State {
        let start = web_sys::Comment::new_with_data("{").unwrap_throw();
        let end = web_sys::Comment::new_with_data("}").unwrap_throw();

        cx.position.insert(&start);
        let state = Box::new(self.inner.build(cx));
        cx.position.insert(&end);

        AnyState { state, start, end }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        match (state.state.deref_mut() as &mut dyn Any)
            .downcast_mut::<V::State>()
        {
            Some(state) => self.inner.rebuild(cx, state),
            None => {
                clear(cx.parent, &state.start, &state.end);

                state.state = Box::new(self.inner.build(BuildCx {
                    position: Position {
                        parent: cx.parent,
                        insert_before: &state.end,
                        waker: cx.waker,
                    },
                }))
            }
        }
    }
}

/// The state for an [`AnyView`].
pub struct AnyState<Output> {
    state: Box<dyn State<Output>>,
    start: web_sys::Comment,
    end: web_sys::Comment,
}

impl<Output: 'static> State<Output> for AnyState<Output> {
    fn run(&mut self, output: &mut Output) {
        self.state.run(output)
    }
}

/// Wraps a [`View`], erasing its [`State`] type.
///
/// Using this inside a [`ravel::with`] callback makes it possible to dynamically
/// choose an implementation type.
pub fn any<V: View, Output: 'static>(view: V) -> AnyView<V, Output> {
    AnyView {
        inner: view,
        phantom: PhantomData,
    }
}
