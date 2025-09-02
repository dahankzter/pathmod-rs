use pathmod::prelude::*;

#[derive(Accessor, Debug, PartialEq)]
struct Foo {
    a: i32,
    b: Bar,
}

#[derive(Accessor, Debug, PartialEq)]
struct Bar {
    x: i32,
}

#[test]
fn named_field_accessors_work() {
    let mut foo = Foo { a: 1, b: Bar { x: 2 } };

    // Access direct field
    let acc_a = Foo::acc_a();
    assert_eq!(*acc_a.get(&foo), 1);
    acc_a.set(&mut foo, 10);
    assert_eq!(foo.a, 10);

    // Compose into nested field
    let acc_b = Foo::acc_b();
    let acc_x = Bar::acc_x();
    let acc_bx = acc_b.compose(acc_x);
    assert_eq!(*acc_bx.get(&foo), 2);
    acc_bx.set_mut(&mut foo, |v| *v += 5);
    assert_eq!(foo.b.x, 7);

    let v = 99;
    acc_bx.set_clone(&mut foo, &v);
    assert_eq!(foo.b.x, 99);
}
