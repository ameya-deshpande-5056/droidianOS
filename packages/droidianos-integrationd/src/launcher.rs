use std::env;

mod waydroid;

pub fn main() {
    let mut args = env::args().skip(1);
    let package = match args.next() {
        Some(value) if !value.is_empty() => value,
        _ => {
            eprintln!("droidianos-launch: missing package name");
            std::process::exit(2);
        }
    };

    if let Err(error) = waydroid::launch_package(&package) {
        eprintln!("droidianos-launch: {}", error);
        std::process::exit(1);
    }
}
