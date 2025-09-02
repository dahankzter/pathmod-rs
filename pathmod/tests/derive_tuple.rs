use pathmod::prelude::*;

#[derive(Accessor, Debug, PartialEq)]
struct Pair(i32, i64);

#[derive(Accessor, Debug, PartialEq)]
struct Wrapper(Pair);

#[test]
fn tuple_field_accessors_work() {
    let mut p = Pair(1, 2);
    let a0 = Pair::acc_0();
    let a1 = Pair::acc_1();

    assert_eq!(*a0.get(&p), 1);
    assert_eq!(*a1.get(&p), 2);

    a0.set(&mut p, 10);
    a1.set_mut(&mut p, |v| *v += 5);
    assert_eq!(p.0, 10);
    assert_eq!(p.1, 7);

    let c = 99;
    a0.set_clone(&mut p, &c);
    assert_eq!(p.0, 99);
}

#[test]
fn compose_through_tuple() {
    let mut w = Wrapper(Pair(3, 4));
    let acc_pair = Wrapper::acc_0();
    let acc_second = Pair::acc_1();
    let acc = acc_pair.compose(acc_second);

    assert_eq!(*acc.get(&w), 4);
    acc.set_mut(&mut w, |v| *v *= 2);
    assert_eq!(w.0 .1, 8);
}
