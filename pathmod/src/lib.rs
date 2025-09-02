//! Pathmod — tiny, composable accessors for ergonomic, type-safe deep mutation.
//!
//! This is the user-facing re-export crate. Depend on `pathmod` and bring the
//! prelude into scope to use both the derive macros and the runtime:
//!
//! ```rust
//! use pathmod::prelude::*;
//!
//! #[derive(Accessor, Debug, PartialEq)]
//! struct Bar { x: i32 }
//!
//! #[derive(Accessor, Debug, PartialEq)]
//! struct Foo { a: i32, b: Bar }
//!
//! let mut foo = Foo { a: 1, b: Bar { x: 2 } };
//! // Access direct field
//! let acc_a = Foo::acc_a();
//! assert_eq!(*acc_a.get(&foo), 1);
//! acc_a.set(&mut foo, 10);
//! assert_eq!(foo.a, 10);
//!
//! // Compose to reach nested leaf Foo -> Bar -> x
//! let acc_bx = Foo::acc_b().compose(Bar::acc_x());
//! acc_bx.set_mut(&mut foo, |v| *v += 5);
//! assert_eq!(foo.b.x, 7);
//! ```
//!
//! See also:
//! - [`pathmod_core`] for the `Accessor` runtime and its operations.
//! - [`pathmod_derive`] for the derive macros generating `acc_*` and `with_*`.

pub use pathmod_core::*;
pub use pathmod_derive::*;

/// Prelude re-exporting the most commonly used items.
///
/// Bring this into scope in your modules:
///
/// ```rust
/// use pathmod::prelude::*;
/// ```
pub mod prelude {
    pub use pathmod_core::prelude::*;
    pub use pathmod_derive::*;
}
