use web_sys::wasm_bindgen::UnwrapThrowExt as _;

use crate::{
    dom::{clear, Position},
    BuildCx, Builder, RebuildCx, State, View, ViewMarker, Web,
};

impl<V: View> Builder<Web> for Option<V> {
    type State = OptionState<V::State>;

    fn build(self, cx: BuildCx) -> Self::State {
        let start = web_sys::Comment::new_with_data("{").unwrap_throw();
        let end = web_sys::Comment::new_with_data("}").unwrap_throw();

        cx.position.insert(&start);
        let state = self.map(|b| b.build(cx));
        cx.position.insert(&end);

        OptionState { state, start, end }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        match (self, &mut state.state) {
            (None, None) => {}
            (None, Some(_)) => {
                state.state = None;
                clear(cx.parent, &state.start, &state.end);
            }
            (Some(b), None) => {
                state.state = Some(b.build(BuildCx {
                    position: Position {
                        parent: cx.parent,
                        insert_before: &state.end,
                        waker: cx.waker,
                    },
                }));
            }
            (Some(b), Some(state)) => b.rebuild(cx, state),
        }
    }
}

/// The state for an [`Option`]al component.
pub struct OptionState<S> {
    state: Option<S>,
    start: web_sys::Comment,
    end: web_sys::Comment,
}

impl<S, Output> State<Output> for OptionState<S>
where
    S: State<Output>,
{
    fn run(&mut self, output: &mut Output) {
        let Some(state) = &mut self.state else { return };
        state.run(output)
    }
}

impl<S> ViewMarker for OptionState<S> {}
