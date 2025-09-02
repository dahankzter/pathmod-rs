// UI tests using trybuild to exercise error arms of the derive macro

#[test]
fn derive_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/non_struct.rs");
    t.compile_fail("tests/ui/unit_struct.rs");
    // Visibility-related error when trying to use accessor on a private type
    t.compile_fail("tests/ui/visibility_private_type.rs");
    // Generics positive case: should compile
    t.pass("tests/ui/generic_ok.rs");
}
