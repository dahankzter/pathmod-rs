Pathmod
=======

[![Crates.io](https://img.shields.io/crates/v/pathmod.svg)](https://crates.io/crates/pathmod)
[![docs.rs](https://img.shields.io/docsrs/pathmod)](https://docs.rs/pathmod)
[![Coveralls](https://coveralls.io/repos/github/dahankzter/pathmod-rs/badge.svg?branch=main)](https://coveralls.io/github/dahankzter/pathmod-rs?branch=main)

Pathmod — derive tiny, composable accessors for ergonomic, type-safe deep mutation.

Overview
- Derive-centric API: #[derive(Accessor)] generates tiny, const accessors for each field (named or tuple) as inherent pub methods on your type. Use the single pathmod crate (re-export) for a smooth UX.
- Composition-first ergonomics: Chain accessors to focus deeply nested fields without boilerplate using Accessor::compose.
- Zero-copy projection: Accessors are Copy and represented as a byte offset from the root type to the field; composition is O(1) offset addition.
- Safe surface: All public APIs are safe; unsafe is contained inside the core crate.

Why? (Rationale)
Rust encourages explicit ownership and borrowing, but it can be verbose to tunnel through nested structs just to tweak a leaf field. Pathmod gives you tiny, composable “field lenses” you can derive and chain:
- Less boilerplate: No need to hand-write getters/mutators for each field.
- Expressive composition: Build Accessor<T, U>.compose(Accessor<U, V>) -> Accessor<T, V> to target deep fields.
- Minimal overhead: Accessor is just an isize offset and is Copy. Composing accessors adds offsets.
- Clear semantics for cloning: set_clone clones only the target field value (MVP). The root type does not need Clone.

Crates in this workspace
- pathmod: Re-export crate; depend on this in your projects. Crates.io: https://crates.io/crates/pathmod
- pathmod_core: Core Accessor runtime (offset-based, safe API). Crates.io: https://crates.io/crates/pathmod_core
- pathmod_derive: Proc-macro derive that generates const accessors using core::mem::offset_of!. Crates.io: https://crates.io/crates/pathmod_derive

Minimum Supported Rust Version (MSRV)
- MSRV: 1.89+ (required by core::mem::offset_of!). CI enforces building, linting, and testing on Rust 1.89.0 and stable.

Quick start
Add dependency (when published this will be on crates.io; in this repo use the workspace):
- use pathmod::prelude::*; // Bring Accessor derive and Accessor into scope

Examples
1) Named-field structs
```rust
use pathmod::prelude::*;

#[derive(Accessor, Debug, PartialEq)]
struct Bar { x: i32 }

#[derive(Accessor, Debug, PartialEq)]
struct Foo { a: i32, b: Bar }

fn main() {
    let mut foo = Foo { a: 1, b: Bar { x: 2 } };

    // Access a direct field
    let acc_a = Foo::acc_a();
    assert_eq!(*acc_a.get(&foo), 1);
    acc_a.set(&mut foo, 10);
    assert_eq!(foo.a, 10);

    // Compose into a nested field
    let acc_b = Foo::acc_b();
    let acc_x = Bar::acc_x();
    let acc_bx = acc_b.compose(acc_x);

    assert_eq!(*acc_bx.get(&foo), 2);
    acc_bx.set_mut(&mut foo, |v| *v += 5);
    assert_eq!(foo.b.x, 7);

    // MVP clone semantics: only V: Clone is required
    let v = 99;
    acc_bx.set_clone(&mut foo, &v);
    assert_eq!(foo.b.x, 99);
}
```

2) Tuple structs
```rust
use pathmod::prelude::*;

#[derive(Accessor, Debug, PartialEq)]
struct Pair(i32, i64);

fn main() {
    let mut p = Pair(1, 2);
    let a0 = Pair::acc_0();
    let a1 = Pair::acc_1();

    assert_eq!(*a0.get(&p), 1);
    assert_eq!(*a1.get(&p), 2);

    a0.set(&mut p, 10);
    a1.set_mut(&mut p, |v| *v += 5);
    assert_eq!(p.0, 10);
    assert_eq!(p.1, 7);
}
```

3) Deeper nested example
```rust
use pathmod::prelude::*;

#[derive(Accessor, Debug, PartialEq)]
struct Address { city: String, zip: u32 }

#[derive(Accessor, Debug, PartialEq)]
struct Profile { address: Address, stats: Stats }

#[derive(Accessor, Debug, PartialEq)]
struct Stats { logins: u32 }

#[derive(Accessor, Debug, PartialEq)]
struct User { profile: Profile, settings: Settings }

#[derive(Accessor, Debug, PartialEq)]
struct Settings { theme: Theme }

#[derive(Accessor, Debug, PartialEq)]
struct Theme { name: String }

fn main() {
    let mut u = User {
        profile: Profile {
            address: Address { city: "berlin".into(), zip: 10115 },
            stats: Stats { logins: 0 },
        },
        settings: Settings { theme: Theme { name: "light".into() } },
    };

    // Compose across 3 hops: User -> Profile -> Address -> city:String
    let acc_city = User::acc_profile()
        .compose(Profile::acc_address())
        .compose(Address::acc_city());

    assert_eq!(acc_city.get(&u).as_str(), "berlin");

    // In-place deep mutation
    acc_city.set_mut(&mut u, |c| c.make_ascii_uppercase());
    assert_eq!(u.profile.address.city, "BERLIN");

    // Deep set via cloning just the leaf value
    let new_city = String::from("Lund");
    acc_city.set_clone(&mut u, &new_city);
    assert_eq!(u.profile.address.city, "Lund");

    // Reuse composition: other deep path (theme name)
    let acc_theme_name = User::acc_settings()
        .compose(Settings::acc_theme())
        .compose(Theme::acc_name());

    acc_theme_name.set(&mut u, "dark".to_string());
    assert_eq!(u.settings.theme.name, "dark");

    // Compose different leaves and use independently
    let acc_zip = User::acc_profile()
        .compose(Profile::acc_address())
        .compose(Address::acc_zip());
    acc_zip.set_mut(&mut u, |z| *z += 5);
    assert_eq!(u.profile.address.zip, 10120);
}
```

Derive-centric API
- Add #[derive(Accessor)] to your struct. For each field, the macro generates:
  - pub const accessor methods on the type (acc_<field>() / acc_<index>()).
  - reconstruction helpers: with_<field>(self, new: FieldTy) -> Self (or with_<index> for tuple fields) that consume self and return a new value with only that field replaced. These helpers move (not clone) other fields, enabling minimal-clone reconstruction.
- Named fields: acc_<field>() -> Accessor<Self, FieldTy>
- Tuple fields: acc_<index>() -> Accessor<Self, FieldTy>
- Bring the API into scope with use pathmod::prelude::*.

Composition
- Compose accessors to focus deep fields: Accessor<T, U>.compose(Accessor<U, V>) -> Accessor<T, V>.
- Composition is O(1) because accessors are just byte offsets; composing adds offsets.
- You can keep and reuse composed accessors, e.g., let acc = A::acc_b().compose(B::acc_c());

API sketch
- Accessor<T, F>::get(&T) -> &F
- Accessor<T, F>::get_mut(&mut T) -> &mut F
- Accessor<T, F>::set(&mut T, F)
- Accessor<T, F>::set_mut(&mut T, impl FnOnce(&mut F))
- Accessor<T, F>::set_clone(&mut T, &F) where F: Clone
- Accessor<T, F>::compose(self, Accessor<F, V>) -> Accessor<T, V>

Design notes
- Representation: Accessor stores the byte offset from &T to &F. get/get_mut compute the field pointer via pointer arithmetic (unsafe internally, safe API externally).
- Derive: #[derive(Accessor)] generates, for each field, a pub const fn acc_<field>() -> Accessor<Self, FieldTy> (or acc_<idx> for tuple fields) using offset_of!.
- Composition: Offsets add. Accessor<T,U>.compose(Accessor<U,V>) = Accessor<T,V> with combined offset.
- Clone semantics (MVP): set_clone clones the provided &F and writes it into the field. Only F: Clone is required; T does not need Clone. This property holds through composition.

Visibility
- The derive generates inherent accessor methods on your type: pub const fn acc_<field>() -> Accessor<Self, FieldTy> (or acc_<idx> for tuple fields).
- These methods are pub on the impl block, but Rust’s normal visibility rules still apply:
  - If the type itself is not visible, callers outside its module cannot reference it or its methods.
  - If a field type is private to a module, you cannot name it from outside even if the accessor method exists.
- Our UI tests include examples where calling an accessor on a private type from outside its module fails with the expected E0603 error.

Limitations and roadmap
- UI diagnostics for complex generics/visibility: planned.
- Minimal-clone reconstruction along the path with precise Clone bounds.
- Indexing builders (e.g., acc_items_at(idx) for Vec/arrays).
- Enum support via a dedicated derive (e.g., #[derive(EnumAccess)]).

Development
- Run tests: cargo test
- Coverage:
  - make coverage-summary  # prints text summary for all crates
  - make coverage          # generates HTML at target/llvm-cov/html/index.html

Single-version policy and Release (tag-based, crates.io)
- All crates in this workspace share a single version number that corresponds to the git tag (vX.Y.Z).
- Tag-only flow (no local version bumping required):
  1) Ensure the repository secret CRATES_IO_TOKEN is set in GitHub (from your crates.io account). The workflow maps CRATES_IO_TOKEN to CARGO_REGISTRY_TOKEN for cargo publish.
  2) Create and push a tag vX.Y.Z (e.g., git tag v0.1.0 && git push origin v0.1.0).
  3) The Release workflow runs cargo-release, which sets all crate versions to X.Y.Z in CI and publishes in order: pathmod_derive -> pathmod_core -> pathmod.
- Notes:
  - Local Cargo.toml versions can be placeholders; CI updates them from the tag during the release job.
  - If you prefer manual bumps, the Makefile still provides: make set-version v=X.Y.Z (optional).

Note about what appears in the coverage summary
- The terminal table lists files that have executable coverage regions. Our pathmod crate is a thin re-export crate; its src/lib.rs only re-exports items, so it typically has 0 instrumentable lines and may not appear in the table.
- We still run tests from the pathmod crate to validate the re-exported API and macro usage across crates; the executed code lives in pathmod_core (runtime) and pathmod_derive (macro expansion), so coverage is attributed to those crates’ source files.
- If you want the re-export crate to show up explicitly in the table, add any small executable item (e.g., a trivial function) and a test calling it so it has at least one instrumented line.

License
Dual-licensed under MIT or Apache-2.0.
