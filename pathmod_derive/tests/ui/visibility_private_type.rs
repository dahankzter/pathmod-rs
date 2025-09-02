mod m {
    use pathmod::prelude::*;
    #[derive(Accessor)]
    struct Private {
        field: i32,
    }

    pub fn try_access_private() {
        // Even though the accessor methods are pub, the type is private to this module.
        // Returning the accessor or calling it from outside should not be possible; however,
        // this inner function compiles. The error will be when an outer module tries to use it.
        let _ = Private::acc_field();
    }
}

fn main() {
    // Attempt to call the generated accessor on a private type from outside its module.
    // This should fail visibility checks.
    let _ = m::Private::acc_field();
}
