Pathmod
=======

Pathmod — derive tiny, composable accessors for ergonomic, type-safe deep mutation.

Overview
- Derive-first API: #[derive(Accessor)] generates tiny accessors for each field.
- Compose accessors: Chain accessors to focus deeply nested fields without boilerplate.
- Zero-copy projection: Accessors are Copy, represented as a byte offset from the root type to the field.
- Safe surface: All public APIs are safe; unsafe is contained inside the core crate.
- Re-export crate: Use the single pathmod crate for a smooth user experience.

Why? (Rationale)
Rust encourages explicit ownership and borrowing, but it can be verbose to tunnel through nested structs just to tweak a leaf field. Pathmod gives you tiny, composable “field lenses” you can derive and chain:
- Less boilerplate: No need to hand-write getters/mutators for each field.
- Expressive composition: Build Accessor<T, U>.compose(Accessor<U, V>) -> Accessor<T, V> to target deep fields.
- Minimal overhead: Accessor is just an isize offset and is Copy. Composing accessors adds offsets.
- Clear semantics for cloning: set_clone clones only the target field value (MVP). The root type does not need Clone.

Crates in this workspace
- pathmod: Re-export crate; depend on this in your projects.
- pathmod_core: Core Accessor runtime (offset-based, safe API).
- pathmod_derive: Proc-macro derive that generates const accessors using core::mem::offset_of!.

Minimum Supported Rust Version (MSRV)
- Intended MSRV: 1.89+ (for core::mem::offset_of!). If your toolchain is older, update Rust.

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
- Generated accessor methods are inherent impls on your type and are always pub in the current MVP. Future versions may add visibility control.

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

Note about what appears in the coverage summary
- The terminal table lists files that have executable coverage regions. Our pathmod crate is a thin re-export crate; its src/lib.rs only re-exports items, so it typically has 0 instrumentable lines and may not appear in the table.
- We still run tests from the pathmod crate to validate the re-exported API and macro usage across crates; the executed code lives in pathmod_core (runtime) and pathmod_derive (macro expansion), so coverage is attributed to those crates’ source files.
- If you want the re-export crate to show up explicitly in the table, add any small executable item (e.g., a trivial function) and a test calling it so it has at least one instrumented line.

License
Dual-licensed under MIT or Apache-2.0.
