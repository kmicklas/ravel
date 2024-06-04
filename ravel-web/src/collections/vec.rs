use std::marker::PhantomData;

use ravel::{with, Token};
use web_sys::wasm_bindgen::UnwrapThrowExt;

use crate::{
    dom::{clear, Position},
    BuildCx, Builder, Cx, RebuildCx, State, Web,
};

pub struct SliceBuilder<'data, T, RenderItem, S> {
    data: &'data [T],
    render_item: RenderItem,
    phantom: PhantomData<S>,
}

impl<'data, T, RenderItem, S: 'static> Builder<Web>
    for SliceBuilder<'data, T, RenderItem, S>
where
    RenderItem: Fn(Cx<S, Web>, usize, &T) -> Token<S>,
{
    type State = SliceState<S>;

    fn build(self, cx: BuildCx) -> Self::State {
        let data = self
            .data
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let header =
                    web_sys::Comment::new_with_data("|").unwrap_throw();
                cx.position.insert(&header);

                Entry {
                    header,
                    state: with(|cx| (self.render_item)(cx, i, v)).build(cx),
                }
            })
            .collect();

        let footer = web_sys::Comment::new_with_data("|").unwrap_throw();
        cx.position.insert(&footer);

        SliceState { data, footer }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        for (i, (v, entry)) in
            self.data.iter().zip(state.data.iter_mut()).enumerate()
        {
            with(|cx| (self.render_item)(cx, i, v))
                .rebuild(cx, &mut entry.state)
        }

        if self.data.len() > state.data.len() {
            state.data.extend(
                self.data
                    .into_iter()
                    .enumerate()
                    .skip(state.data.len())
                    .map(|(i, v)| {
                        let position = Position {
                            parent: cx.parent,
                            insert_before: &state.footer,
                            waker: cx.waker,
                        };

                        let header =
                            web_sys::Comment::new_with_data("|").unwrap_throw();
                        position.insert(&header);

                        Entry {
                            header,
                            state: with(|cx| (self.render_item)(cx, i, v))
                                .build(BuildCx { position }),
                        }
                    }),
            )
        } else if self.data.len() < state.data.len() {
            clear(
                &cx.parent,
                &state.data[self.data.len()].header,
                &state.footer,
            );
            state.data.truncate(self.data.len());
        }
    }
}

pub struct SliceState<S> {
    data: Vec<Entry<S>>,
    footer: web_sys::Comment,
}

impl<S, Output> State<Output> for SliceState<S>
where
    S: State<Output>,
{
    fn run(&mut self, output: &mut Output) {
        for entry in self.data.iter_mut() {
            entry.state.run(output);
        }
    }
}

struct Entry<S> {
    header: web_sys::Comment,
    state: S,
}

pub fn slice<T, RenderItem, S>(
    data: &[T],
    render_item: RenderItem,
) -> SliceBuilder<T, RenderItem, S>
where
    RenderItem: Fn(Cx<S, Web>, usize, &T) -> Token<S>,
{
    SliceBuilder {
        render_item,
        data,
        phantom: PhantomData,
    }
}
