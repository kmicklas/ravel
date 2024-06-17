use std::{
    cmp::Ordering, collections::BTreeMap, marker::PhantomData, ops::Bound,
};

use ravel::{with, State, Token};
use web_sys::wasm_bindgen::UnwrapThrowExt;

use crate::{
    dom::{clear, Position},
    BuildCx, Builder, Cx, RebuildCx, Web,
};

pub struct BTreeMapBuilder<'data, K, V, RenderItem, S> {
    data: &'data BTreeMap<K, V>,
    render_item: RenderItem,
    phantom: PhantomData<S>,
}

impl<'data, K: 'static + Clone + Ord, V, RenderItem, S: 'static> Builder<Web>
    for BTreeMapBuilder<'data, K, V, RenderItem, S>
where
    RenderItem: Fn(Cx<S, Web>, &K, &V) -> Token<S>,
{
    type State = BTreeMapState<K, S>;

    fn build(self, cx: BuildCx) -> Self::State {
        let data = self
            .data
            .iter()
            .map(|(k, v)| {
                let header =
                    web_sys::Comment::new_with_data("|").unwrap_throw();
                cx.position.insert(&header);

                (
                    k.clone(),
                    Entry {
                        header,
                        state: with(|cx| (self.render_item)(cx, k, v))
                            .build(cx),
                    },
                )
            })
            .collect();

        let footer = web_sys::Comment::new_with_data("|").unwrap_throw();
        cx.position.insert(&footer);

        BTreeMapState { data, footer }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        let sdata = &mut state.data;
        let mut source = self.data.iter().peekable();
        let mut existing = sdata.iter_mut().peekable();

        let mut add = vec![];
        let mut remove = vec![];

        loop {
            match (source.peek(), existing.peek()) {
                (None, None) => break,
                (None, Some((_, _))) => {
                    let (k, _) = existing.next().unwrap();
                    remove.push(k.clone());
                }
                (Some((_, _)), None) => {
                    let (k, v) = source.next().unwrap();

                    let position = Position {
                        parent: cx.parent,
                        insert_before: &state.footer,
                        waker: cx.waker,
                    };

                    let header =
                        web_sys::Comment::new_with_data("|").unwrap_throw();
                    position.insert(&header);

                    add.push((
                        k.clone(),
                        Entry {
                            header,
                            state: with(|cx| (self.render_item)(cx, k, v))
                                .build(BuildCx { position }),
                        },
                    ));
                }
                (Some((sk, _)), Some((ek, _))) => match sk.cmp(ek) {
                    Ordering::Equal => {
                        let (sk, sv) = source.next().unwrap();
                        let (_, e) = existing.next().unwrap();
                        with(|cx| (self.render_item)(cx, sk, sv))
                            .rebuild(cx, &mut e.state)
                    }
                    Ordering::Less => todo!(),
                    Ordering::Greater => {
                        let (ek, _) = existing.next().unwrap();
                        remove.push(ek.clone());
                    }
                },
            }
        }

        sdata.extend(add);
        for k in remove {
            let e = sdata.remove(&k).unwrap();
            let end = match sdata
                .range((Bound::Excluded(&k), Bound::Unbounded))
                .next()
            {
                Some((_, e)) => &e.header,
                None => &state.footer,
            };

            clear(cx.parent, &e.header, end);
            cx.parent.remove_child(&e.header).unwrap_throw();
        }
    }
}

pub struct BTreeMapState<K, S> {
    data: BTreeMap<K, Entry<S>>,
    footer: web_sys::Comment,
}

impl<K: 'static + Ord, S, Output> State<Output> for BTreeMapState<K, S>
where
    S: State<Output>,
{
    fn run(&mut self, output: &mut Output) {
        for entry in self.data.values_mut() {
            entry.state.run(output);
        }
    }
}

struct Entry<S> {
    header: web_sys::Comment,
    state: S,
}

pub fn btree_map<K: Ord, V, RenderItem, S>(
    data: &BTreeMap<K, V>,
    render_item: RenderItem,
) -> BTreeMapBuilder<K, V, RenderItem, S>
where
    RenderItem: Fn(Cx<S, Web>, &K, &V) -> Token<S>,
{
    BTreeMapBuilder {
        render_item,
        data,
        phantom: PhantomData,
    }
}
