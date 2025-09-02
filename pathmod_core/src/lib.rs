#![doc = "Core types for pathmod"]

use core::marker::PhantomData;

/// A small, copyable accessor that focuses into a field F inside a root T.
///
/// This runtime representation carries two function pointers: one for shared
/// access and one for mutable access. All other operations build on these.
#[derive(Debug, Clone, Copy)]
pub struct Accessor<T, F> {
    get_ref: fn(&T) -> &F,
    get_mut: fn(&mut T) -> &mut F,
    _phantom: PhantomData<(T, F)>,
}

impl<T, F> Accessor<T, F> {
    /// Create a new accessor from two functions.
    ///
    /// Safety: The provided functions must consistently refer to the same field
    /// in both shared and mutable forms. They must not alias distinct fields.
    pub const fn new(get_ref: fn(&T) -> &F, get_mut: fn(&mut T) -> &mut F) -> Self {
        Self { get_ref, get_mut, _phantom: PhantomData }
    }

    /// Borrow the focused field immutably.
    pub fn get<'a>(&self, root: &'a T) -> &'a F {
        (self.get_ref)(root)
    }

    /// Borrow the focused field mutably.
    pub fn get_mut<'a>(&self, root: &'a mut T) -> &'a mut F {
        (self.get_mut)(root)
    }

    /// Set by moving a new value into the focused location.
    pub fn set(&self, root: &mut T, value: F) {
        *self.get_mut(root) = value;
    }

    /// Mutate the focused location in-place using the provided closure.
    pub fn set_mut(&self, root: &mut T, f: impl FnOnce(&mut F)) {
        f(self.get_mut(root));
    }

    /// Set by cloning the provided value into the focused location.
    ///
    /// MVP semantics: the caller provides a shared reference to the value, and
    /// we clone it at the top and move it into the field. This requires F: Clone
    /// on the value type only; no Clone bound is required on T.
    pub fn set_clone(&self, root: &mut T, value: &F)
    where
        F: Clone,
    {
        *self.get_mut(root) = value.clone();
    }
}

pub mod prelude {
    pub use crate::Accessor;
}
