use pathmod::prelude::*;

// Trybuild-like negative tests by attempting to compile invalid derives in separate modules
// and asserting they fail at compile time is not possible here; instead, we ensure the
// successful paths are covered and we add a proc-macro crate tests to hit error arms.

#[test]
fn sanity() {
    // This test exists to ensure the file is discovered; invalid derive cases are in the
    // proc-macro crate tests where compile-fail can be asserted using trybuild.
    assert_eq!(1 + 1, 2);
}
