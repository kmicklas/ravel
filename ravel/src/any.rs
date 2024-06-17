use std::any::Any;

/// Trait for upcasting to [`Any`], implemented automatically.
///
/// This is a workaround until `trait_upcasting` is stabilized.
pub trait AsAny: Any {
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any {
        self
    }
}
