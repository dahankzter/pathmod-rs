#![doc = "Core types for pathmod"]

/// Placeholder for core Accessor type to be implemented in later todos.
/// For now we expose a minimal stub so the workspace compiles.
#[derive(Debug, Clone, Copy)]
pub struct Accessor<T, F> {
    _phantom: core::marker::PhantomData<(T, F)>,
}

impl<T, F> Default for Accessor<T, F> {
    fn default() -> Self {
        Self { _phantom: core::marker::PhantomData }
    }
}

pub mod prelude {
    pub use crate::Accessor;
}
