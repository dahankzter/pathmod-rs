use pathmod::prelude::*;

#[derive(Accessor, Debug, PartialEq)]
struct Address {
    city: String,
    zip: u32,
}

#[derive(Accessor, Debug, PartialEq)]
struct Profile {
    address: Address,
    stats: Stats,
}

#[derive(Accessor, Debug, PartialEq)]
struct Stats {
    logins: u32,
}

#[derive(Accessor, Debug, PartialEq)]
struct User {
    profile: Profile,
    settings: Settings,
}

#[derive(Accessor, Debug, PartialEq)]
struct Settings {
    theme: Theme,
}

#[derive(Accessor, Debug, PartialEq)]
struct Theme {
    name: String,
}

#[test]
fn minimal_clone_reconstruction_bottom_up() {
    let u = User {
        profile: Profile {
            address: Address {
                city: "berlin".into(),
                zip: 10115,
            },
            stats: Stats { logins: 3 },
        },
        settings: Settings {
            theme: Theme {
                name: "light".into(),
            },
        },
    };

    // Rebuild only the path User -> Profile -> Address -> city with a new String,
    // moving all other fields (no Clone needed anywhere).
    let profile = u.profile; // move out of u
    let settings = u.settings; // move out of u

    let address = profile.address; // move out of profile
    let stats = profile.stats; // move out of profile

    let address2 = address.with_city("Lund".to_string());
    let profile2 = Profile {
        address: address2,
        stats,
    };
    let u2 = User {
        profile: profile2,
        settings,
    };

    assert_eq!(u2.profile.address.city, "Lund");
    assert_eq!(u2.profile.address.zip, 10115);
    assert_eq!(u2.profile.stats.logins, 3);
    assert_eq!(u2.settings.theme.name, "light");
}
