use std::io;
use std::process::Command;

#[derive(Clone)]
struct App {
    source: String,
    name: String,
    id: String,
}

fn main() {
    if let Err(error) = run() {
        show_error("Software Center", &error.to_string());
        eprintln!("droidianos-software-center: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    loop {
        let action = choose_action()?;
        match action.as_deref() {
            Some("Installed applications") => show_installed_apps()?,
            Some("Search system packages") => search_apt_packages()?,
            Some("Search Flatpak applications") => search_flatpak_apps()?,
            Some("Update Flatpak applications") => update_flatpak_apps()?,
            Some("Install APK file") => install_apk_file()?,
            Some("Quit") | None => return Ok(()),
            Some(_) => {}
        }
    }
}

fn choose_action() -> io::Result<Option<String>> {
    zenity_output(
        Command::new("zenity")
            .args([
                "--list",
                "--title",
                "Software Center",
                "--width",
                "520",
                "--height",
                "320",
                "--column",
                "Action",
                "Installed applications",
                "Search system packages",
                "Search Flatpak applications",
                "Update Flatpak applications",
                "Install APK file",
                "Quit",
            ]),
    )
}

fn show_installed_apps() -> io::Result<()> {
    let apps = installed_apps();
    if apps.is_empty() {
        show_info("Installed Applications", "No installed applications were found.");
        return Ok(());
    }

    let mut command = Command::new("zenity");
    command.args([
        "--list",
        "--title",
        "Installed Applications",
        "--width",
        "900",
        "--height",
        "600",
        "--print-column",
        "ALL",
        "--separator",
        "\t",
        "--column",
        "Source",
        "--column",
        "Name",
        "--column",
        "ID",
    ]);
    for app in &apps {
        command.arg(&app.source).arg(&app.name).arg(&app.id);
    }

    let selected = match zenity_output(&mut command)? {
        Some(value) => value,
        None => return Ok(()),
    };
    let selected_app = match selected_app(&apps, &selected) {
        Some(app) => app,
        None => return Ok(()),
    };

    if confirm(
        "Remove Application",
        &format!("Remove {}?\n\nSource: {}\nID: {}", selected_app.name, selected_app.source, selected_app.id),
    )? {
        remove_app(&selected_app)?;
    }

    Ok(())
}

fn installed_apps() -> Vec<App> {
    let mut apps = Vec::new();
    apps.extend(installed_apt_apps());
    apps.extend(installed_flatpak_apps());
    apps.extend(installed_android_apps());
    apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    apps
}

fn installed_apt_apps() -> Vec<App> {
    let output = Command::new("dpkg-query")
        .args(["-W", "-f=${binary:Package}\t${Version}\n"])
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut apps = Vec::new();

    for line in stdout.lines().take(500) {
        let mut parts = line.split('\t');
        let package = match parts.next() {
            Some(package) if !package.is_empty() => package,
            _ => continue,
        };
        apps.push(App {
            source: "deb".to_string(),
            name: package.to_string(),
            id: package.to_string(),
        });
    }

    apps
}

fn installed_flatpak_apps() -> Vec<App> {
    let output = Command::new("flatpak")
        .args(["list", "--app", "--columns=application,name"])
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut apps = Vec::new();

    for line in stdout.lines() {
        let mut parts = line.split('\t');
        let id = match parts.next() {
            Some(id) if !id.is_empty() => id,
            _ => continue,
        };
        let name = parts.next().filter(|name| !name.is_empty()).unwrap_or(id);
        apps.push(App {
            source: "flatpak".to_string(),
            name: name.to_string(),
            id: id.to_string(),
        });
    }

    apps
}

fn installed_android_apps() -> Vec<App> {
    let output = Command::new("droidianos-integrationd").arg("--list").output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let registry = String::from_utf8_lossy(&output.stdout);
    android_apps_from_registry(&registry)
}

fn android_apps_from_registry(registry: &str) -> Vec<App> {
    let mut apps = Vec::new();
    let mut remainder = registry;

    while let Some(start) = remainder.find('{') {
        remainder = &remainder[start + 1..];
        let end = match remainder.find('}') {
            Some(end) => end,
            None => break,
        };
        let object = &remainder[..end];
        remainder = &remainder[end + 1..];

        let package = match json_string_field(object, "package") {
            Some(package) => package,
            None => continue,
        };
        let name = json_string_field(object, "name").unwrap_or_else(|| package.clone());
        apps.push(App {
            source: "apk".to_string(),
            name,
            id: package,
        });
    }

    apps
}

fn json_string_field(object: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", field);
    let start = object.find(&pattern)? + pattern.len();
    let mut value = String::new();
    let mut escaped = false;

    for character in object[start..].chars() {
        if escaped {
            value.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some(value);
        } else {
            value.push(character);
        }
    }

    None
}

fn selected_app(apps: &[App], selected: &str) -> Option<App> {
    let mut parts = selected.split('\t');
    let source = parts.next()?;
    let _name = parts.next()?;
    let id = parts.next()?;

    apps.iter()
        .find(|app| app.source == source && app.id == id)
        .cloned()
}

fn remove_app(app: &App) -> io::Result<()> {
    let status = match app.source.as_str() {
        "apk" => Command::new("waydroid").args(["app", "remove", &app.id]).status()?,
        "flatpak" => Command::new("flatpak").args(["uninstall", "-y", &app.id]).status()?,
        _ => Command::new("pkexec")
            .args(["apt-get", "remove", "-y", &app.id])
            .status()?,
    };

    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "remove failed"));
    }

    if app.source == "apk" {
        let _ = Command::new("droidianos-integrationd").arg("--refresh").status();
    }

    Ok(())
}

fn search_apt_packages() -> io::Result<()> {
    let query = match entry("Search system packages", "Search term:")? {
        Some(query) if !query.trim().is_empty() => query,
        _ => return Ok(()),
    };
    let results = apt_search(&query);
    if results.is_empty() {
        show_info("Search Results", "No packages found.");
        return Ok(());
    }

    let mut command = Command::new("zenity");
    command.args([
        "--list",
        "--title",
        "Search Results",
        "--width",
        "900",
        "--height",
        "600",
        "--print-column",
        "1",
        "--column",
        "Package",
        "--column",
        "Description",
    ]);
    for (package, description) in &results {
        command.arg(package).arg(description);
    }

    let package = match zenity_output(&mut command)? {
        Some(package) => package,
        None => return Ok(()),
    };

    if confirm("Install Package", &format!("Install {}?", package))? {
        let status = Command::new("pkexec")
            .args(["apt-get", "install", "-y", &package])
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "install failed"));
        }
    }

    Ok(())
}

fn apt_search(query: &str) -> Vec<(String, String)> {
    let output = Command::new("apt-cache").args(["search", query]).output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines().take(200) {
        let mut parts = line.splitn(2, " - ");
        let package = match parts.next() {
            Some(package) if !package.is_empty() => package,
            _ => continue,
        };
        let description = parts.next().unwrap_or("");
        results.push((package.to_string(), description.to_string()));
    }

    results
}

fn search_flatpak_apps() -> io::Result<()> {
    let query = match entry("Search Flatpak applications", "Search term:")? {
        Some(query) if !query.trim().is_empty() => query,
        _ => return Ok(()),
    };
    let results = flatpak_search(&query);
    if results.is_empty() {
        show_info("Search Results", "No Flatpak applications found.");
        return Ok(());
    }

    let mut command = Command::new("zenity");
    command.args([
        "--list",
        "--title",
        "Flatpak Search Results",
        "--width",
        "900",
        "--height",
        "600",
        "--print-column",
        "1",
        "--column",
        "Application ID",
        "--column",
        "Name",
        "--column",
        "Description",
    ]);
    for app in &results {
        command.arg(&app.id).arg(&app.name).arg(&app.source);
    }

    let app_id = match zenity_output(&mut command)? {
        Some(app_id) => app_id,
        None => return Ok(()),
    };

    if confirm("Install Flatpak", &format!("Install {}?", app_id))? {
        let status = Command::new("flatpak")
            .args(["install", "-y", "flathub", &app_id])
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Flatpak install failed"));
        }
    }

    Ok(())
}

fn flatpak_search(query: &str) -> Vec<App> {
    let output = Command::new("flatpak")
        .args(["search", "--columns=application,name,description", query])
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines().take(100) {
        let mut parts = line.split('\t');
        let id = match parts.next() {
            Some(id) if !id.is_empty() => id,
            _ => continue,
        };
        let name = parts.next().filter(|name| !name.is_empty()).unwrap_or(id);
        let description = parts.next().unwrap_or("");
        results.push(App {
            source: description.to_string(),
            name: name.to_string(),
            id: id.to_string(),
        });
    }

    results
}

fn update_flatpak_apps() -> io::Result<()> {
    if !confirm("Update Flatpak Applications", "Update all Flatpak applications?")? {
        return Ok(());
    }

    let status = Command::new("flatpak").args(["update", "-y"]).status()?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Flatpak update failed"));
    }

    show_info("Update Flatpak Applications", "Flatpak applications updated.");
    Ok(())
}

fn install_apk_file() -> io::Result<()> {
    let path = match zenity_output(
        Command::new("zenity").args([
            "--file-selection",
            "--title",
            "Install APK",
            "--file-filter",
            "APK files | *.apk",
        ]),
    )? {
        Some(path) => path,
        None => return Ok(()),
    };

    let status = Command::new("droidianos-apk-installer").arg(path).status()?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "APK install failed"));
    }

    Ok(())
}

fn entry(title: &str, text: &str) -> io::Result<Option<String>> {
    zenity_output(Command::new("zenity").args(["--entry", "--title", title, "--text", text]))
}

fn confirm(title: &str, text: &str) -> io::Result<bool> {
    let status = Command::new("zenity")
        .args(["--question", "--title", title, "--text", text])
        .status()?;
    Ok(status.success())
}

fn show_info(title: &str, text: &str) {
    let _ = Command::new("zenity")
        .args(["--info", "--title", title, "--text", text])
        .status();
}

fn show_error(title: &str, text: &str) {
    let _ = Command::new("zenity")
        .args(["--error", "--title", title, "--text", text])
        .status();
}

fn zenity_output(command: &mut Command) -> io::Result<Option<String>> {
    let output = command.output()?;
    if !output.status.success() {
        return Ok(None);
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}
