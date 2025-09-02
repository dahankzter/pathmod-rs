use pathmod::prelude::*;

// Enum is not supported by #[derive(Accessor)]
#[derive(Accessor)]
enum E { A }

fn main() {}
