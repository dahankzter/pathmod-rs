pathmod_core crate
==================

[![Crates.io](https://img.shields.io/crates/v/pathmod_core.svg)](https://crates.io/crates/pathmod_core)
[![docs.rs](https://img.shields.io/docsrs/pathmod_core)](https://docs.rs/pathmod_core)

Purpose
- Core runtime for Pathmod. Provides the Accessor<T, F> type and its safe methods for reading/mutating focused fields.
- Used indirectly by most users via the re-export crate `pathmod`, but library authors may depend directly if they need only the runtime.

What is Accessor?
- A tiny, Copy value that stores the byte offset from &T to &F.
- Operations:
  - get(&T) -> &F
  - get_mut(&mut T) -> &mut F
  - set(&mut T, F)
  - set_mut(&mut T, impl FnOnce(&mut F))
  - set_clone(&mut T, &F) where F: Clone (MVP semantics: only the leaf value is cloned)
  - compose(self, Accessor<F, V>) -> Accessor<T, V>
- Representation: offset-based; composition is O(1) addition of offsets. Public API is safe; unsafe is encapsulated inside.

Construction
- Usually constructed by the derive macro from `pathmod_derive` (via `#[derive(Accessor)]`).
- Also provides:
  - const unsafe fn from_offset(isize) for macro/const construction.
  - fn from_fns(get_ref, get_mut) for runtime construction (computes offset safely without dereferencing).

When should I depend on this crate directly?
- If you are building your own derive/projection utilities or want to avoid a proc-macro dependency and construct accessors at runtime.
- Otherwise, prefer `pathmod` which re-exports everything.

MSRV
- Intended MSRV 1.89+ (matches usage of core::mem::offset_of! in the derive crate).

License
Dual-licensed under MIT or Apache-2.0.
