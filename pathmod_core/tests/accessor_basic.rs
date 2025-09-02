use pathmod_core::Accessor;

#[derive(Debug, PartialEq)]
struct Inner { x: i32 }

#[derive(Debug, PartialEq)]
struct Outer { inner: Inner }

// Helper field accessors constructed manually for tests.
fn acc_inner() -> Accessor<Outer, Inner> {
    fn get_ref(o: &Outer) -> &Inner { &o.inner }
    fn get_mut(o: &mut Outer) -> &mut Inner { &mut o.inner }
    Accessor::from_fns(get_ref, get_mut)
}

fn acc_x() -> Accessor<Inner, i32> {
    fn get_ref(i: &Inner) -> &i32 { &i.x }
    fn get_mut(i: &mut Inner) -> &mut i32 { &mut i.x }
    Accessor::from_fns(get_ref, get_mut)
}

#[test]
fn get_and_get_mut_work() {
    let a = acc_inner();
    let o = Outer { inner: Inner { x: 7 } };
    assert_eq!(a.get(&o).x, 7);
}

#[test]
fn set_and_set_mut_and_set_clone_work() {
    let a = acc_inner();
    let b = acc_x();

    let mut o = Outer { inner: Inner { x: 0 } };

    // set via x accessor
    b.set(&mut o.inner, 5);
    assert_eq!(o.inner.x, 5);

    // mutate via closure
    b.set_mut(&mut o.inner, |v| *v += 2);
    assert_eq!(o.inner.x, 7);

    // set_clone
    let v = 42;
    b.set_clone(&mut o.inner, &v);
    assert_eq!(o.inner.x, 42);
}

#[test]
fn compose_and_deep_update_work() {
    let a = acc_inner();
    let b = acc_x();
    let ab: Accessor<Outer, i32> = a.compose(b);

    let mut o = Outer { inner: Inner { x: 1 } };
    assert_eq!(*ab.get(&o), 1);

    ab.set(&mut o, 10);
    assert_eq!(o.inner.x, 10);

    ab.set_mut(&mut o, |v| *v += 5);
    assert_eq!(o.inner.x, 15);

    let v = 22;
    ab.set_clone(&mut o, &v);
    assert_eq!(o.inner.x, 22);
}
