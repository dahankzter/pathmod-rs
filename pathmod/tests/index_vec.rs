use pathmod::prelude::*;

#[derive(Accessor, Debug, PartialEq)]
struct Bag {
    items: Vec<i32>,
}

#[derive(Accessor, Debug, PartialEq)]
struct Wrapper {
    bag: Bag,
}

#[test]
fn index_into_vec_in_place_and_clone() {
    let mut w = Wrapper { bag: Bag { items: vec![1, 2, 3] } };

    let acc_items = Wrapper::acc_bag().compose(Bag::acc_items());

    // Read element at index 1
    assert_eq!(*acc_items.get_at(&w, 1), 2);

    // In-place mutate element at index 2
    acc_items.set_mut_at(&mut w, 2, |v| *v += 10);
    assert_eq!(w.bag.items, vec![1, 2, 13]);

    // Set by moving
    acc_items.set_at(&mut w, 0, 99);
    assert_eq!(w.bag.items, vec![99, 2, 13]);

    // Set via clone
    let x = 77;
    acc_items.set_clone_at(&mut w, 1, &x);
    assert_eq!(w.bag.items, vec![99, 77, 13]);
}
