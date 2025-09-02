# TODO

- [x] Initialize a clean workspace centered on a derive-first design (crates: pathmod_core, pathmod_derive, pathmod re-export).
- [x] Implement pathmod_core::Accessor<T, F> runtime with set, set_mut, set_clone, get, get_mut.
- Implement Accessor::compose to chain Accessor<T, U> -> Accessor<U, V> into Accessor<T, V>.
- Implement #[derive(Accessor)] for named-field structs, generating pub const acc_<field>() -> Accessor<Self, FieldTy>.
- Implement #[derive(Accessor)] for tuple structs, generating pub const acc_<index>() -> Accessor<Self, FieldTy>.
- Generate set_clone using top-level clone (MVP) and document Clone requirements on root type.
- Add unit tests covering get/get_mut/set/set_mut/set_clone and composed deep updates.
- Add trybuild UI tests for invalid derives (non-struct targets), generics bounds, and visibility diagnostics.
- Add an example demonstrating derive-based deep mutation via composed accessors.
- Update README to present the derive-centric API, composition, and visibility rules.
- Set up CI (fmt, clippy -D warnings, tests including doctests) and enforce/document MSRV 1.89+.
- Roadmap: implement minimal-clone reconstruction along the path with precise Clone bounds.
- Roadmap: add indexing builders (e.g., acc_items_at(idx) -> Accessor<Self, Item>) for Vec/arrays.
- Roadmap: add #[derive(EnumAccess)] for ergonomic enum variant setters/getters.
