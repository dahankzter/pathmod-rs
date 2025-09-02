pathmod_derive crate
====================

[![Crates.io](https://img.shields.io/crates/v/pathmod_derive.svg)](https://crates.io/crates/pathmod_derive)
[![docs.rs](https://img.shields.io/docsrs/pathmod_derive)](https://docs.rs/pathmod_derive)

Purpose
- Proc-macro crate providing `#[derive(Accessor)]` for structs (named and tuple).
- Generates inherent `pub const` accessor methods for each field:
  - Named fields: `acc_<field>() -> pathmod::Accessor<Self, FieldTy>`
  - Tuple fields: `acc_<idx>() -> pathmod::Accessor<Self, FieldTy>`

Behavior
- Accessors are built using `core::mem::offset_of!`, enabling `const` construction.
- Composition is available via the runtime type (Accessor) from the core crate.
- Unit structs and non-struct targets are rejected with clear compile errors.

How to use
- Most users should depend on `pathmod` and `use pathmod::prelude::*;` which re-exports this derive.
- If depending directly, add to Cargo.toml and import the macro: `use pathmod_derive::Accessor;` then annotate your types.

Example
See top-level README for end-to-end examples. Minimal snippet:

```rust
use pathmod::prelude::*;

#[derive(Accessor)]
struct Address { city: String }

fn example() {
    let _acc = Address::acc_city();
}
```

Limitations and diagnostics
- Unit structs are not supported.
- Enums are not supported (future roadmap may add separate enum accessor derive).
- Visibility follows Rust rules: even though generated methods are `pub`, private types/field types aren’t accessible from outside their module.
- UI tests with `trybuild` cover error messages and generics visibility cases.

MSRV
- Intended MSRV 1.89+ (for `core::mem::offset_of!`).

License
Dual-licensed under MIT or Apache-2.0.
