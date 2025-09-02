Pathmod crate
==============

Purpose
- This is the user-facing, re-export crate for Pathmod. Depend on this crate in your application/library to use the derive and runtime with a single import path.
- It re-exports:
  - pathmod_core::Accessor and its prelude
  - pathmod_derive::Accessor (the derive macro)

Quick use
- Bring the API into scope:
  - use pathmod::prelude::*;
- Derive and compose accessors:
  - #[derive(Accessor)] on your structs to get pub const acc_<field>() (or acc_<idx>() for tuple fields).
  - Compose with Accessor::compose for deep focus.

Example
See the top-level README.md for complete, runnable examples, including deep composition. A tiny sketch:

```rust
use pathmod::prelude::*;

#[derive(Accessor)]
struct Inner { x: i32 }

#[derive(Accessor)]
struct Outer { inner: Inner }

fn demo(mut o: Outer) {
    let acc = Outer::acc_inner().compose(Inner::acc_x());
    acc.set_mut(&mut o, |v| *v += 1);
}
```

When should I depend on this crate?
- In almost all cases. It provides a smooth developer experience by consolidating the runtime type and the derive macro under one path.

Notes
- This crate intentionally contains minimal code (mostly re-exports). Coverage tools may omit it from summaries because it has few/no instrumentable lines. Tests still run against it to validate the public UX.
- MSRV: 1.89+ (via core::mem::offset_of! used by the derive). Use a newer toolchain if necessary.

License
Dual-licensed under MIT or Apache-2.0.
