use std::{iter::once, marker::PhantomData};

use ravel::{with, State, Token};
use web_sys::wasm_bindgen::UnwrapThrowExt;

use crate::{
    dom::{clear, Position},
    BuildCx, Builder, Cx, RebuildCx, Web,
};

pub struct IterBuilder<I, RenderItem, S> {
    iter: I,
    render_item: RenderItem,
    phantom: PhantomData<S>,
}

impl<I: Iterator, RenderItem, S: 'static> Builder<Web>
    for IterBuilder<I, RenderItem, S>
where
    RenderItem: Fn(Cx<S, Web>, usize, I::Item) -> Token<S>,
{
    type State = IterState<S>;

    fn build(self, cx: BuildCx) -> Self::State {
        let data = self
            .iter
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

        IterState { data, footer }
    }

    fn rebuild(mut self, cx: RebuildCx, state: &mut Self::State) {
        let mut data = state.data.iter_mut();

        for i in 0.. {
            match (self.iter.next(), data.next()) {
                (None, None) => break,
                (None, Some(entry)) => {
                    clear(cx.parent, &entry.header, &state.footer);
                    state.data.truncate(i);
                    break;
                }
                (Some(v), None) => {
                    state.data.extend(once(v).chain(self.iter).map(|v| {
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
                    }));
                    break;
                }
                (Some(v), Some(entry)) => {
                    with(|cx| (self.render_item)(cx, i, v))
                        .rebuild(cx, &mut entry.state)
                }
            }
        }
    }
}

pub struct IterState<S> {
    data: Vec<Entry<S>>,
    footer: web_sys::Comment,
}

impl<S, Output> State<Output> for IterState<S>
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

pub fn iter<I: IntoIterator, RenderItem, S>(
    iter: I,
    render_item: RenderItem,
) -> IterBuilder<I::IntoIter, RenderItem, S>
where
    RenderItem: Fn(Cx<S, Web>, usize, I::Item) -> Token<S>,
{
    IterBuilder {
        render_item,
        iter: iter.into_iter(),
        phantom: PhantomData,
    }
}
