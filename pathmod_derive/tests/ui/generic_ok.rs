use pathmod::prelude::*;

#[derive(Accessor)]
struct Wrapper<T> { value: T }

fn main() {
    let w = Wrapper { value: 1u32 };
    let acc = Wrapper::<u32>::acc_value();
    let _ = acc.get(&w);
}
