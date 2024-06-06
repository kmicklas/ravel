//! HTML elements.

use std::marker::PhantomData;

use web_sys::wasm_bindgen::{JsValue, UnwrapThrowExt};

use crate::{
    dom::Position, BuildCx, Builder, RebuildCx, State, ViewMarker, Web,
};

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
        build_el(cx, Kind::NAME, self.body)
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

fn build_el<Body: Builder<Web>>(
    cx: BuildCx,
    kind: &'static str,
    body: Body,
) -> ElState<Body::State> {
    let el = gloo_utils::document().create_element(kind).unwrap_throw();

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
    ($name:ident, $t:ident) => {
        #[doc = concat!("`", stringify!($name), "` element.")]
        #[repr(transparent)]
        #[derive(Copy, Clone)]
        pub struct $t<Body>(pub Body);

        impl<Body: Builder<Web>> Builder<Web> for $t<Body> {
            type State = ElState<Body::State>;

            fn build(self, cx: BuildCx) -> Self::State {
                build_el(cx, stringify!($name), self.0)
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

        #[doc = concat!("`", stringify!($name), "` element.")]
        pub fn $name<Body>(body: Body) -> $t<Body> {
            $t(body)
        }
    };
}

make_el!(a, A);
make_el!(b, B);
make_el!(button, Button);
make_el!(div, Div);
make_el!(footer, Footer);
make_el!(form, Form);
make_el!(h1, H1);
make_el!(h2, H2);
make_el!(h3, H3);
make_el!(h4, H4);
make_el!(h5, H5);
make_el!(h6, H6);
make_el!(header, Header);
make_el!(input, Input);
make_el!(label, Label);
make_el!(li, Li);
make_el!(p, P);
make_el!(section, Section);
make_el!(span, Span);
make_el!(strong, Strong);
make_el!(table, Table);
make_el!(tbody, TBody);
make_el!(thead, THead);
make_el!(td, Td);
make_el!(tr, Tr);
make_el!(ul, Ul);
