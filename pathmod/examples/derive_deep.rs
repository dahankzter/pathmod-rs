use pathmod::prelude::*;

// A small, realistic nested model
#[derive(Accessor, Debug, PartialEq)]
struct Address { city: String, zip: u32 }

#[derive(Accessor, Debug, PartialEq)]
struct Profile { address: Address, stats: Stats }

#[derive(Accessor, Debug, PartialEq)]
struct Stats { logins: u32 }

#[derive(Accessor, Debug, PartialEq)]
struct User { profile: Profile, settings: Settings }

#[derive(Accessor, Debug, PartialEq)]
struct Settings { theme: Theme }

#[derive(Accessor, Debug, PartialEq)]
struct Theme { name: String }

fn main() {
    let mut user = User {
        profile: Profile {
            address: Address { city: "berlin".into(), zip: 10115 },
            stats: Stats { logins: 1 },
        },
        settings: Settings { theme: Theme { name: "light".into() } },
    };

    // Compose a deep accessor: User -> Profile -> Address -> city
    let acc_city = User::acc_profile()
        .compose(Profile::acc_address())
        .compose(Address::acc_city());

    println!("city before: {}", acc_city.get(&user));

    // In-place deep mutation via set_mut
    acc_city.set_mut(&mut user, |c| c.make_ascii_uppercase());
    println!("city upper: {}", acc_city.get(&user));

    // Set via cloning only the leaf value
    let new_city = String::from("Lund");
    acc_city.set_clone(&mut user, &new_city);
    println!("city after set_clone: {}", acc_city.get(&user));

    // Compose another deep accessor: User -> Settings -> Theme -> name
    let acc_theme_name = User::acc_settings()
        .compose(Settings::acc_theme())
        .compose(Theme::acc_name());

    acc_theme_name.set(&mut user, "dark".to_string());
    println!("theme after set: {}", acc_theme_name.get(&user));

    // Show a non-string leaf with arithmetic updates
    let acc_zip = User::acc_profile()
        .compose(Profile::acc_address())
        .compose(Address::acc_zip());

    acc_zip.set_mut(&mut user, |z| *z += 5);
    println!("zip after +5: {}", acc_zip.get(&user));
}
