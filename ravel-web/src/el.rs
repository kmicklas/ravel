//! HTML elements.

use std::marker::PhantomData;

use ravel::State;
use web_sys::wasm_bindgen::{JsValue, UnwrapThrowExt};

use crate::{dom::Position, BuildCx, Builder, RebuildCx, ViewMarker, Web};

/// Trait to identify element types.
pub trait ElKind: 'static {
    /// The name of the element.
    const NAME: &'static str;
}

/// An arbitrary element.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct El<Kind: ElKind, Body> {
    kind: PhantomData<Kind>,
    body: Body,
}

impl<Kind: ElKind, Body: Builder<Web>> Builder<Web> for El<Kind, Body> {
    type State = ElState<Body::State>;

    fn build(self, cx: BuildCx) -> Self::State {
        build_el(cx, create_element(Kind::NAME), self.body)
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        self.body.rebuild(
            RebuildCx {
                parent: &state.node,
                waker: cx.waker,
            },
            &mut state.body,
        )
    }
}

/// The state of an [`El`].
pub struct ElState<S> {
    node: web_sys::Element,
    body: S,
}

impl<Output, S> State<Output> for ElState<S>
where
    S: State<Output>,
{
    fn run(&mut self, output: &mut Output) {
        self.body.run(output)
    }
}

impl<S> ViewMarker for ElState<S> {}

/// An arbitrary element.
pub fn el<Kind: ElKind, Body>(_: Kind, body: Body) -> El<Kind, Body> {
    El {
        kind: PhantomData,
        body,
    }
}

fn create_element(kind: &'static str) -> web_sys::Element {
    gloo_utils::document().create_element(kind).unwrap_throw()
}

fn build_el<Body: Builder<Web>>(
    cx: BuildCx,
    el: web_sys::Element,
    body: Body,
) -> ElState<Body::State> {
    let state = body.build(BuildCx {
        position: Position {
            parent: &el,
            insert_before: &JsValue::NULL.into(),
            waker: cx.position.waker,
        },
    });

    cx.position.insert(&el);

    ElState {
        body: state,
        node: el,
    }
}

macro_rules! make_el {
    ($name:ident, $t:ident, $create:expr) => {
        #[doc = concat!("`", stringify!($name), "` element.")]
        #[repr(transparent)]
        #[derive(Copy, Clone)]
        pub struct $t<Body>(pub Body);

        impl<Body: Builder<Web>> Builder<Web> for $t<Body> {
            type State = ElState<Body::State>;

            fn build(self, cx: BuildCx) -> Self::State {
                build_el(cx, $create, self.0)
            }

            fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
                self.0.rebuild(
                    RebuildCx {
                        parent: &state.node,
                        waker: cx.waker,
                    },
                    &mut state.body,
                )
            }
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/el_gen.rs"));
