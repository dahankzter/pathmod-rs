#![doc = r#"Pathmod Core — runtime types for composable field accessors

This crate provides the runtime `Accessor<T, F>` type used by the derive macros in
`pathmod_derive` and re-exported by the `pathmod` crate. An `Accessor<T, F>` is a
small, `Copy` value that focuses into a field `F` within a root `T`.

Highlights
- Zero-cost composition: `Accessor` stores a byte offset from `&T` to `&F`; composing
  accessors is O(1) offset addition.
- Safe surface: All public APIs are safe. Internals use pointer arithmetic; constructors
  compute valid offsets for you.
- Clear clone semantics (MVP): `set_clone` only requires `F: Clone` and does not require
  `T: Clone`, even when composed deeply.

Quick example
```rust
use pathmod_core::Accessor;

#[derive(Debug, PartialEq)]
struct Bar { x: i32 }
#[derive(Debug, PartialEq)]
struct Foo { a: i32, b: Bar }

fn acc_b() -> Accessor<Foo, Bar> {
    fn get_ref(f: &Foo) -> &Bar { &f.b }
    fn get_mut(f: &mut Foo) -> &mut Bar { &mut f.b }
    Accessor::from_fns(get_ref, get_mut)
}
fn acc_x() -> Accessor<Bar, i32> {
    fn get_ref(b: &Bar) -> &i32 { &b.x }
    fn get_mut(b: &mut Bar) -> &mut i32 { &mut b.x }
    Accessor::from_fns(get_ref, get_mut)
}

let mut foo = Foo { a: 1, b: Bar { x: 2 } };
let acc = acc_b().compose(acc_x());
acc.set_mut(&mut foo, |v| *v += 5);
assert_eq!(foo.b.x, 7);
```

Safety notes
- Internally, accessors are represented by a byte offset and use unsafe pointer arithmetic
  to project fields. The public API is safe when accessors are constructed by the provided
  derive macros or `from_fns`.

"#]

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
    /// # Safety
    /// The `offset` must satisfy all of the following conditions for every valid instance of `T`:
    /// - It is the exact byte distance from the start of `T` to the target field `F` within the
    ///   same allocation (i.e., derived from an actual field projection of `T`).
    /// - The resulting pointer computed as `(&T as *const u8).offset(offset) as *const F` is
    ///   properly aligned for `F` and points to initialized memory owned by the same `T` object.
    /// - The accessor will only ever be used with values of type `T` that have the same layout
    ///   with respect to the field `F` (e.g., not a different type or transmuted layout).
    ///
    /// Violating any of these preconditions is undefined behavior. Prefer constructing accessors
    /// via `#[derive(Accessor)]` or `Accessor::from_fns`, which compute valid offsets for you.
    pub const unsafe fn from_offset(offset: isize) -> Self {
        Self {
            offset,
            _phantom: PhantomData,
        }
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

/// Indexing operations for accessors that focus `Vec<E>`.
///
/// Provided as a blanket impl for `Accessor<T, Vec<E>>`.
pub trait Indexing<T, E> {
    /// Borrow the element at `idx` immutably.
    ///
    /// ```rust
    /// use pathmod_core::{Accessor, Indexing};
    /// #[derive(Debug)]
    /// struct Bag { items: Vec<i32> }
    /// fn acc_items() -> Accessor<Bag, Vec<i32>> {
    ///     fn gr(b: &Bag) -> &Vec<i32> { &b.items }
    ///     fn gm(b: &mut Bag) -> &mut Vec<i32> { &mut b.items }
    ///     Accessor::from_fns(gr, gm)
    /// }
    /// let b = Bag { items: vec![1,2,3] };
    /// let acc = acc_items();
    /// assert_eq!(*acc.get_at(&b, 1), 2);
    /// ```
    fn get_at<'a>(&self, root: &'a T, idx: usize) -> &'a E;

    /// Borrow the element at `idx` mutably.
    fn get_mut_at<'a>(&self, root: &'a mut T, idx: usize) -> &'a mut E;

    /// Set the element at `idx` by moving `value` in.
    fn set_at(&self, root: &mut T, idx: usize, value: E);

    /// Mutate the element at `idx` in-place using the closure.
    fn set_mut_at(&self, root: &mut T, idx: usize, f: impl FnOnce(&mut E));

    /// Set the element at `idx` by cloning from `value`.
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
