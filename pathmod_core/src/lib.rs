#![doc = "Core types for pathmod\n\nThis crate provides the runtime `Accessor<T, F>` type used by the derive macros in `pathmod_derive` and re-exported by the `pathmod` crate.\n\nClone requirements for set_clone (MVP):\n- `Accessor::set_clone(&mut T, &F)` clones the provided value at the top level and writes it into the focused field.\n- Only `F: Clone` is required. The root type `T` does not need to implement `Clone` for this operation.\n- When composing accessors (e.g., `Accessor<T, U>.compose(Accessor<U, V>)`), calling `set_clone` on the composed accessor still only requires `V: Clone`.\n\nSafety notes:\n- Internally, accessors are represented by a byte offset and use unsafe pointer arithmetic to project fields. The public API is safe when accessors are constructed by the provided derive macros or `from_fns`.\n"]

use core::marker::PhantomData;

/// A small, copyable accessor that focuses into a field F inside a root T.
///
/// Representation: a byte offset from the start of T to the field F. This
/// allows cheap composition by offset addition. All operations are implemented
/// via unsafe pointer arithmetic but expose a safe API.
#[derive(Debug, Clone, Copy)]
pub struct Accessor<T, F> {
    /// Byte offset from a T pointer to its field F.
    offset: isize,
    _phantom: PhantomData<fn(T) -> F>,
}

impl<T, F> Accessor<T, F> {
    /// Construct from a precomputed byte offset.
    ///
    /// Safety: `offset` must be the correct byte distance from `&T` to `&F`
    /// within the same allocation for any valid instance of `T`.
    pub const unsafe fn from_offset(offset: isize) -> Self {
        Self { offset, _phantom: PhantomData }
    }

    /// Runtime constructor from field-selection functions. Computes the offset
    /// using raw pointer projection without dereferencing invalid memory.
    pub fn from_fns(get_ref: fn(&T) -> &F, _get_mut: fn(&mut T) -> &mut F) -> Self {
        // Create an arbitrary base pointer; using null is fine since we don't deref.
        let base = core::ptr::null::<T>();
        // Obtain the address of the projected field via the provided getter by
        // transmuting it to a raw-pointer based projection.
        // We rely on Rust layout and that the getter returns a direct reference
        // into the same object (a field).
        unsafe fn to_raw<T, F>(f: fn(&T) -> &F) -> fn(*const T) -> *const F {
            // Transmute of function pointer types with compatible ABI.
            core::mem::transmute::<fn(&T) -> &F, fn(*const T) -> *const F>(f)
        }
        let raw_get: fn(*const T) -> *const F = unsafe { to_raw(get_ref) };
        let field_ptr = raw_get(base);
        let offset = (field_ptr as isize) - (base as isize);
        // Safety: the offset was computed from a field projection function.
        unsafe { Accessor::from_offset(offset) }
    }

    /// Borrow the focused field immutably.
    pub fn get<'a>(&self, root: &'a T) -> &'a F {
        unsafe {
            let base = root as *const T as *const u8;
            let ptr = base.offset(self.offset) as *const F;
            &*ptr
        }
    }

    /// Borrow the focused field mutably.
    pub fn get_mut<'a>(&self, root: &'a mut T) -> &'a mut F {
        unsafe {
            let base = root as *mut T as *mut u8;
            let ptr = base.offset(self.offset) as *mut F;
            &mut *ptr
        }
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
    /// MVP semantics:
    /// - The caller provides a shared reference to the value, and we perform a top-level
    ///   `Clone` of that value, then move it into the field.
    /// - Only `F: Clone` is required; the root type `T` does not need to implement `Clone`.
    /// - This behavior composes: for a composed accessor focusing `T -> ... -> V`, calling
    ///   `set_clone` only requires `V: Clone`.
    pub fn set_clone(&self, root: &mut T, value: &F)
    where
        F: Clone,
    {
        *self.get_mut(root) = value.clone();
    }

    /// Compose this accessor with another, yielding an accessor from T to V.
    ///
    /// Given `self: Accessor<T, U>` and `next: Accessor<U, V>`, returns
    /// `Accessor<T, V>` that focuses by first going through `self` then `next`.
    pub fn compose<V>(self, next: Accessor<F, V>) -> Accessor<T, V> {
        // Offsets add: T -> F, then F -> V.
        let offset = self.offset + next.offset;
        unsafe { Accessor::from_offset(offset) }
    }
}

pub trait Indexing<T, E> {
    fn get_at<'a>(&self, root: &'a T, idx: usize) -> &'a E;
    fn get_mut_at<'a>(&self, root: &'a mut T, idx: usize) -> &'a mut E;
    fn set_at(&self, root: &mut T, idx: usize, value: E);
    fn set_mut_at(&self, root: &mut T, idx: usize, f: impl FnOnce(&mut E));
    fn set_clone_at(&self, root: &mut T, idx: usize, value: &E)
    where
        E: Clone;
}

impl<T, E> Indexing<T, E> for Accessor<T, Vec<E>> {
    fn get_at<'a>(&self, root: &'a T, idx: usize) -> &'a E {
        &self.get(root)[idx]
    }
    fn get_mut_at<'a>(&self, root: &'a mut T, idx: usize) -> &'a mut E {
        &mut self.get_mut(root)[idx]
    }
    fn set_at(&self, root: &mut T, idx: usize, value: E) {
        self.get_mut(root)[idx] = value;
    }
    fn set_mut_at(&self, root: &mut T, idx: usize, f: impl FnOnce(&mut E)) {
        f(&mut self.get_mut(root)[idx]);
    }
    fn set_clone_at(&self, root: &mut T, idx: usize, value: &E)
    where
        E: Clone,
    {
        self.get_mut(root)[idx] = value.clone();
    }
}

pub mod prelude {
    pub use crate::Accessor;
    pub use crate::Indexing;
}
