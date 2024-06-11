//! Run an event loop for a top-level component.
use std::sync::Arc;

use atomic_waker::AtomicWaker;
use ravel::{with, Builder, Float, State, Token};
use web_sys::wasm_bindgen::JsValue;

use crate::{dom::Position, BuildCx, Cx, RebuildCx, Web};

/// Runs a component on an arbitrary [`web_sys::Element`].
///
/// The `render` callback has read-only access to the `Data`. Due to limitations
/// of Rust's type system, you cannot pass a [`trait@crate::View`] directly
/// here. Instead, the callback must use [`Cx::build`].
///
/// The `sync` callback can update the `Data`, and optionally return [`Some`]
/// value which aborts the event loop.
///
/// The event loop repeats the following steps:
///
/// 1. `render` the `Data`.
/// 1. Suspend the `async` task until awoken.
/// 1. `sync` the `Data` (for example, write updates to an external data store).
pub async fn run<Data, Sync, Render, S, R>(
    parent: &web_sys::Element,
    data: &mut Float<Data>,
    mut sync: Sync,
    mut render: Render,
) -> R
where
    S: State<Data>,
    Sync: FnMut(&mut Data) -> Option<R>,
    Render: FnMut(Cx<S, Web>, &Data) -> Token<S>,
{
    let waker = &Arc::new(AtomicWaker::new());
    waker.register(&futures_micro::waker().await);

    let mut state =
        with(|cx| render(cx, data.as_ref().unwrap())).build(BuildCx {
            position: Position {
                parent,
                insert_before: &JsValue::NULL.into(),
                waker,
            },
        });

    loop {
        futures_micro::sleep().await;

        state.run(data);
        if let Some(result) = sync(data.as_mut().unwrap()) {
            return result;
        }

        with(|cx| render(cx, data.as_ref().unwrap()))
            .rebuild(RebuildCx { parent, waker }, &mut state);

        waker.register(&futures_micro::waker().await);
    }
}

/// Spawns a component in the HTML `<body>` in a new [`wasm_bindgen_futures`]
/// task.
///
/// This is a convenience wrapper around [`run`], to run a complete application,
/// which will never abort.
pub fn spawn_body<Data: 'static, Sync, Render, S>(
    data: Data,
    mut sync: Sync,
    render: Render,
) where
    S: State<Data>,
    Sync: 'static + FnMut(&mut Data),
    Render: 'static + FnMut(Cx<S, Web>, &Data) -> Token<S>,
{
    let body = gloo_utils::body();
    wasm_bindgen_futures::spawn_local(async move {
        let mut data = Float::new(data);
        run(
            &body,
            &mut data,
            move |data| {
                sync(data);
                None
            },
            render,
        )
        .await
    });
}
