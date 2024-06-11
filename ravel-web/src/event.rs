//! HTML events.

use std::{cell::RefCell, marker::PhantomData, ops::DerefMut, rc::Rc};

use ravel::{Float, State};
use web_sys::wasm_bindgen::JsValue;

use crate::{BuildCx, Builder, RebuildCx, Web};

/// Trait to identify event types.
pub trait EventKind: 'static {
    /// The name of the event.
    const NAME: &'static str;

    /// Active events may use [`web_sys::Event::prevent_default`]. By default,
    /// this is
    /// [disabled to improve performance](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#passive).
    const ACTIVE: bool = false;
}

/// An "active" version of an [`EventKind`], which may use
/// [`web_sys::Event::prevent_default`].
///
/// This wraps another kind, setting [`EventKind::ACTIVE`] to `true`.
pub struct Active<K: EventKind>(pub K);

impl<K: EventKind> EventKind for Active<K> {
    const NAME: &'static str = K::NAME;
    const ACTIVE: bool = true;
}

/// An event handler.
pub struct On<Kind: EventKind, Action> {
    action: Action,
    kind: PhantomData<Kind>,
}

impl<Kind: EventKind, Action: 'static> Builder<Web> for On<Kind, Action> {
    type State = OnState<Action>;

    fn build(self, cx: BuildCx) -> Self::State {
        let waker = cx.position.waker.clone();

        let cell = EventCell::new();

        OnState {
            event: cell.clone(),
            _handle: gloo_events::EventListener::new_with_options(
                cx.position.parent,
                Kind::NAME,
                gloo_events::EventListenerOptions {
                    passive: !Kind::ACTIVE,
                    ..Default::default()
                },
                move |e| {
                    cell.put(e.clone());
                    waker.wake();
                },
            ),
            action: self.action,
        }
    }

    fn rebuild(self, _: RebuildCx, state: &mut Self::State) {
        state.action = self.action;
    }
}

/// The state of an [`On`].
pub struct OnState<Action> {
    event: EventCell,
    _handle: gloo_events::EventListener,
    action: Action,
}

impl<Action: 'static + FnMut(&mut Output, web_sys::Event), Output: 'static>
    State<Output> for OnState<Action>
{
    fn run(&mut self, output: &mut Float<Output>) {
        let event = self.event.take();
        if !event.is_null() {
            (self.action)(output.as_mut().unwrap(), event);
        }
    }
}

/// An event handler.
pub fn on<
    Kind: EventKind,
    Action: 'static + FnMut(&mut Output, web_sys::Event),
    Output: 'static,
>(
    _: Kind,
    action: Action,
) -> On<Kind, Action> {
    On {
        action,
        kind: PhantomData,
    }
}

/// An event handler, which does not need access to the [`web_sys::Event`] data.
pub fn on_<
    Kind: EventKind,
    Action: 'static + FnMut(&mut Output),
    Output: 'static,
>(
    _: Kind,
    mut action: Action,
) -> On<Kind, impl 'static + FnMut(&mut Output, web_sys::Event)> {
    On {
        action: move |o: &mut _, _: _| action(o),
        kind: PhantomData,
    }
}

#[derive(Clone)]
struct EventCell(Rc<RefCell<web_sys::Event>>);

impl EventCell {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(JsValue::NULL.into())))
    }

    fn take(&self) -> web_sys::Event {
        std::mem::replace(self.0.borrow_mut().deref_mut(), JsValue::NULL.into())
    }

    fn put(&self, event: web_sys::Event) {
        let event = std::mem::replace(self.0.borrow_mut().deref_mut(), event);
        debug_assert!(event.is_null())
    }
}

macro_rules! make_event {
    ($name:ident, $t:ident) => {
        #[doc = concat!("`", stringify!($name), "` event.")]
        #[derive(Copy, Clone)]
        pub struct $t;

        impl EventKind for $t {
            const NAME: &'static str = stringify!($name);
        }
    };
}

make_event!(dblclick, DblClick);
make_event!(click, Click);
make_event!(input, InputEvent);
make_event!(submit, Submit);
