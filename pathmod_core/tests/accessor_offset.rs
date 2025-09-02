use pathmod_core::{Accessor, Indexing};

#[derive(Debug, PartialEq)]
struct Simple {
    a: u8,
    b: i32,
}

#[derive(Debug, PartialEq)]
struct WithVec {
    items: Vec<i32>,
}

#[test]
fn construct_with_from_offset_and_access() {
    // Compute the offset to field `b`
    let off = core::mem::offset_of!(Simple, b) as isize;
    // SAFETY: `off` is derived from the field offset within `Simple`.
    let acc_b: Accessor<Simple, i32> = unsafe { Accessor::from_offset(off) };

    let mut s = Simple { a: 5, b: 10 };
    assert_eq!(*acc_b.get(&s), 10);

    // Mutate via get_mut to verify the pointer arithmetic path
    *acc_b.get_mut(&mut s) += 7;
    assert_eq!(s.b, 17);
}

#[test]
fn indexing_vec_get_mut_at_is_exercised() {
    // Build an accessor focusing the Vec field using from_offset
    let off = core::mem::offset_of!(WithVec, items) as isize;
    let acc_items: Accessor<WithVec, Vec<i32>> = unsafe { Accessor::from_offset(off) };

    let mut w = WithVec {
        items: vec![10, 20, 30],
    };

    // Read element at index 1 using get_at
    assert_eq!(*acc_items.get_at(&w, 1), 20);

    // Mutate element at index 2 using get_mut_at explicitly
    let m = acc_items.get_mut_at(&mut w, 2);
    *m += 100;
    assert_eq!(w.items, vec![10, 20, 130]);
}
