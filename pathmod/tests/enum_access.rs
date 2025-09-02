use pathmod::prelude::*;

#[derive(EnumAccess, Debug, PartialEq)]
enum Msg {
    Int(i32),
    Text(String),
}

#[test]
fn enum_access_tuple_single_field_variants() {
    let mut m = Msg::Int(5);
    assert!(m.is_int());
    assert_eq!(*m.as_int().unwrap(), 5);
    assert!(m.as_text().is_none());

    m.map_int(|v| *v += 10);
    assert_eq!(m, Msg::Int(15));

    m.set_text("hi".to_string());
    assert!(m.is_text());
    assert_eq!(m.as_text().unwrap(), "hi");
    assert!(m.as_int().is_none());
}
