use std::sync::Arc;

use atomic_waker::AtomicWaker;
use web_sys::wasm_bindgen::UnwrapThrowExt;

#[derive(Copy, Clone)]
pub struct Position<'cx> {
    pub parent: &'cx web_sys::Element,
    pub insert_before: &'cx web_sys::Node,
    // TODO: Remove double pointer.
    pub waker: &'cx Arc<AtomicWaker>,
}

impl Position<'_> {
    pub fn insert(&self, node: &web_sys::Node) {
        self.parent
            .insert_before(node, Some(self.insert_before))
            .unwrap_throw();
    }
}

pub fn clear(
    parent: &web_sys::Node,
    start: &web_sys::Node,
    end: &web_sys::Node,
) {
    while let Some(next) = start.next_sibling() {
        if &next == end {
            break;
        }
        parent.remove_child(&next).unwrap_throw();
    }
}
