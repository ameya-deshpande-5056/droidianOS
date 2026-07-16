use std::env;

fn main() {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("usage: droidianos-apk-inspect <file.apk|file.apks|file.apkm>");
            std::process::exit(2);
        }
    };

    match droidianos_apk::inspect_package(&path) {
        Ok(metadata) => println!("{}", metadata.to_json()),
        Err(error) => {
            eprintln!("droidianos-apk-inspect: {}", error);
            std::process::exit(1);
        }
    }
}
