use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    if let Err(error) = run() {
        show_error("Settings", &error.to_string());
        eprintln!("droidianos-settings: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    loop {
        let action = choose_action()?;
        match action.as_deref() {
            Some("Application permissions") => permissions_page()?,
            Some("Storage and shared folders") => storage_page()?,
            Some("Diagnostics") => diagnostics_page()?,
            Some("Quit") | None => return Ok(()),
            Some(_) => {}
        }
    }
}

fn choose_action() -> io::Result<Option<String>> {
    zenity_output(Command::new("zenity").args([
        "--list",
        "--title",
        "Settings",
        "--width",
        "520",
        "--height",
        "320",
        "--column",
        "Section",
        "Application permissions",
        "Storage and shared folders",
        "Diagnostics",
        "Quit",
    ]))
}

fn permissions_page() -> io::Result<()> {
    let package = match choose_android_package()? {
        Some(package) => package,
        None => return Ok(()),
    };
    let permission = match choose_permission()? {
        Some(permission) => permission,
        None => return Ok(()),
    };
    let state = match choose_permission_state()? {
        Some(state) => state,
        None => return Ok(()),
    };

    start_permissions_service();
    let status = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.droidianos.Permissions",
            "--object-path",
            "/org/droidianos/Permissions",
            "--method",
            "org.droidianos.Permissions.SetPermission",
            &package,
            &permission,
            &state,
        ])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "permission update failed"));
    }

    show_info("Application permissions", "Permission updated.");
    Ok(())
}

fn choose_android_package() -> io::Result<Option<String>> {
    let output = Command::new("droidianos-integrationd").arg("--list").output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => {
            show_info("Application permissions", "No Android applications were found.");
            return Ok(None);
        }
    };
    let registry = String::from_utf8_lossy(&output.stdout);
    let packages = packages_from_registry(&registry);
    if packages.is_empty() {
        show_info("Application permissions", "No Android applications were found.");
        return Ok(None);
    }

    let mut command = Command::new("zenity");
    command.args([
        "--list",
        "--title",
        "Application permissions",
        "--width",
        "700",
        "--height",
        "500",
        "--print-column",
        "1",
        "--column",
        "Package",
    ]);
    for package in packages {
        command.arg(package);
    }

    zenity_output(&mut command)
}

fn choose_permission() -> io::Result<Option<String>> {
    zenity_output(Command::new("zenity").args([
        "--list",
        "--title",
        "Permission",
        "--width",
        "640",
        "--height",
        "420",
        "--print-column",
        "1",
        "--column",
        "Permission",
        "android.permission.CAMERA",
        "android.permission.RECORD_AUDIO",
        "android.permission.ACCESS_FINE_LOCATION",
        "android.permission.ACCESS_COARSE_LOCATION",
        "android.permission.POST_NOTIFICATIONS",
        "android.permission.READ_CONTACTS",
        "android.permission.READ_EXTERNAL_STORAGE",
        "android.permission.WRITE_EXTERNAL_STORAGE",
    ]))
}

fn choose_permission_state() -> io::Result<Option<String>> {
    zenity_output(Command::new("zenity").args([
        "--list",
        "--title",
        "Permission state",
        "--width",
        "360",
        "--height",
        "260",
        "--print-column",
        "1",
        "--column",
        "State",
        "allowed",
        "denied",
        "default",
    ]))
}

fn storage_page() -> io::Result<()> {
    let folder = match zenity_output(Command::new("zenity").args([
        "--file-selection",
        "--directory",
        "--title",
        "Add shared folder",
    ]))? {
        Some(folder) => folder,
        None => return Ok(()),
    };

    let mut folders = read_shared_folders()?;
    if !folders.iter().any(|existing| existing == &folder) {
        folders.push(folder);
    }
    write_shared_folders(&folders)?;
    show_info("Storage and shared folders", "Shared folder policy updated.");
    Ok(())
}

fn diagnostics_page() -> io::Result<()> {
    let report = diagnostics_report();
    show_text("Diagnostics", &report)
}

fn diagnostics_report() -> String {
    let mut report = String::new();
    report.push_str("droidianOS diagnostics\n\n");
    append_command(&mut report, "droidianos-integrationd --list", &["droidianos-integrationd", "--list"]);
    append_command(&mut report, "systemctl --user status droidianos-permissionsd.service", &["systemctl", "--user", "status", "droidianos-permissionsd.service"]);
    append_command(&mut report, "systemctl --user status droidianos-apk-installerd.service", &["systemctl", "--user", "status", "droidianos-apk-installerd.service"]);
    append_command(&mut report, "waydroid status", &["waydroid", "status"]);
    report
}

fn append_command(report: &mut String, label: &str, command: &[&str]) {
    report.push_str("## ");
    report.push_str(label);
    report.push('\n');
    if command.is_empty() {
        return;
    }

    let output = Command::new(command[0]).args(&command[1..]).output();
    match output {
        Ok(output) => {
            report.push_str(&String::from_utf8_lossy(&output.stdout));
            report.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        Err(error) => {
            report.push_str(&error.to_string());
            report.push('\n');
        }
    }
    report.push('\n');
}

fn packages_from_registry(registry: &str) -> Vec<String> {
    let mut packages = Vec::new();
    let mut remainder = registry;

    while let Some(start) = remainder.find('{') {
        remainder = &remainder[start + 1..];
        let end = match remainder.find('}') {
            Some(end) => end,
            None => break,
        };
        let object = &remainder[..end];
        remainder = &remainder[end + 1..];
        if let Some(package) = json_string_field(object, "package") {
            packages.push(package);
        }
    }

    packages.sort();
    packages.dedup();
    packages
}

fn read_shared_folders() -> io::Result<Vec<String>> {
    let path = shared_folders_path()?;
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    Ok(json_array_strings(&contents, "folders"))
}

fn write_shared_folders(folders: &[String]) -> io::Result<()> {
    let path = shared_folders_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut json = String::from("{\"folders\":[");
    for (index, folder) in folders.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(&escape_json(folder));
        json.push('"');
    }
    json.push_str("]}\n");
    fs::write(path, json)
}

fn shared_folders_path() -> io::Result<PathBuf> {
    Ok(config_home()?.join("droidianos/shared-folders.json"))
}

fn config_home() -> io::Result<PathBuf> {
    let config_home = match env::var_os("XDG_CONFIG_HOME") {
        Some(path) => PathBuf::from(path),
        None => {
            let home = env::var_os("HOME").ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "HOME is not set")
            })?;
            PathBuf::from(home).join(".config")
        }
    };

    Ok(config_home)
}

fn start_permissions_service() {
    let _ = Command::new("systemctl")
        .args(["--user", "start", "droidianos-permissionsd.service"])
        .status();
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

fn json_array_strings(contents: &str, field: &str) -> Vec<String> {
    let pattern = format!("\"{}\":[", field);
    let start = match contents.find(&pattern) {
        Some(start) => start + pattern.len(),
        None => return Vec::new(),
    };
    let end = match contents[start..].find(']') {
        Some(end) => start + end,
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    let mut value = String::new();
    let mut in_string = false;
    let mut escaped = false;

    for character in contents[start..end].chars() {
        if !in_string {
            if character == '"' {
                in_string = true;
                value.clear();
            }
            continue;
        }

        if escaped {
            value.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            in_string = false;
            values.push(value.clone());
        } else {
            value.push(character);
        }
    }

    values
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }

    escaped
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

fn show_text(title: &str, text: &str) -> io::Result<()> {
    let status = Command::new("zenity")
        .args([
            "--text-info",
            "--title",
            title,
            "--width",
            "900",
            "--height",
            "650",
        ])
        .arg("--filename")
        .arg(write_temp_text(text)?)
        .status()?;
    if !status.success() {
        return Ok(());
    }
    Ok(())
}

fn write_temp_text(text: &str) -> io::Result<PathBuf> {
    let mut path = env::temp_dir();
    path.push("droidianos-settings-diagnostics.txt");
    fs::write(&path, text)?;
    Ok(path)
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

