use pathmod_core::Accessor;

#[derive(Debug, PartialEq)]
struct Inner { x: i32 }

#[derive(Debug, PartialEq)]
struct Outer { inner: Inner }

// Helper field accessors constructed manually for tests.
const fn acc_inner() -> Accessor<Outer, Inner> {
    fn get_ref(o: &Outer) -> &Inner { &o.inner }
    fn get_mut(o: &mut Outer) -> &mut Inner { &mut o.inner }
    Accessor::new(get_ref, get_mut)
}

const fn acc_x() -> Accessor<Inner, i32> {
    fn get_ref(i: &Inner) -> &i32 { &i.x }
    fn get_mut(i: &mut Inner) -> &mut i32 { &mut i.x }
    Accessor::new(get_ref, get_mut)
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
