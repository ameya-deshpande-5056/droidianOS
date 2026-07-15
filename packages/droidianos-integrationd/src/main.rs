use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use droidianos_dbus_lite::{Connection, Message};

mod waydroid;

const EMPTY_REGISTRY: &str = "{\"apps\":[]}\n";
const SERVICE_NAME: &str = "org.droidianos.Integration";
const INTEGRATION_INTERFACE: &str = "org.droidianos.Integration";
const OBJECT_PATH: &str = "/org/droidianos/Integration";
const LIST_APPS: &str = "ListApps";
const REFRESH_APPS: &str = "RefreshApps";
const LAUNCH_APP: &str = "LaunchApp";
const UNINSTALL_APP: &str = "UninstallApp";
const APP_STATE_CHANGED: &str = "AppStateChanged";

#[derive(Debug)]
struct App {
    package: String,
    name: String,
    desktop_id: String,
    icon_name: String,
}

fn main() {
    let result = match env::args().nth(1).as_deref() {
        Some("--refresh") => refresh_registry().map(|_| ()),
        Some("--list") => list_registry(),
        Some("--daemon") | None => run_daemon(),
        Some("--help") => {
            print_help();
            Ok(())
        }
        Some(arg) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown argument: {}", arg),
        )),
    };

    if let Err(error) = result {
        eprintln!("droidianos-integrationd: {}", error);
        std::process::exit(1);
    }
}

fn print_help() {
    println!("droidianos-integrationd --daemon");
    println!("droidianos-integrationd --refresh");
    println!("droidianos-integrationd --list");
}

fn run_daemon() -> io::Result<()> {
    if let Err(error) = refresh_registry() {
        eprintln!("droidianos-integrationd: initial refresh failed: {}", error);
    }

    run_dbus_service()
}

fn run_dbus_service() -> io::Result<()> {
    let connection = Connection::session_with_name(SERVICE_NAME)?;

    loop {
        if let Some(message) = connection.next_message(1000) {
            handle_dbus_message(&connection, &message);
        }
    }
}

fn handle_dbus_message(connection: &Connection, message: &Message) {
    if message.is_method(INTEGRATION_INTERFACE, LIST_APPS) {
        match registry_contents() {
            Ok(contents) => connection.send_string_reply(message, &contents),
            Err(error) => connection.send_error_reply(message, &error.to_string()),
        }
        return;
    }

    if message.is_method(INTEGRATION_INTERFACE, REFRESH_APPS) {
        match refresh_registry() {
            Ok(_) => connection.send_empty_reply(message),
            Err(error) => connection.send_error_reply(message, &error.to_string()),
        }
        return;
    }

    if message.is_method(INTEGRATION_INTERFACE, LAUNCH_APP) {
        match message.string_pair_args() {
            Ok((package, _activity)) => {
                if package.is_empty() {
                    connection.send_error_reply(message, "package must not be empty");
                    return;
                }
                send_app_state_changed(connection, &package, "Starting");
                match waydroid::launch_package(&package) {
                    Ok(_) => {
                        send_app_state_changed(connection, &package, "Running");
                        connection.send_empty_reply(message);
                    }
                    Err(error) => {
                        send_app_state_changed(connection, &package, "Failed");
                        connection.send_error_reply(message, &error.to_string());
                    }
                }
            }
            Err(error) => connection.send_error_reply(message, &error.to_string()),
        }
        return;
    }

    if message.is_method(INTEGRATION_INTERFACE, UNINSTALL_APP) {
        connection.send_error_reply(message, "method is not implemented yet");
    }
}

fn send_app_state_changed(connection: &Connection, package: &str, state: &str) {
    connection.send_string_pair_signal(
        OBJECT_PATH,
        INTEGRATION_INTERFACE,
        APP_STATE_CHANGED,
        package,
        state,
    );
}

fn list_registry() -> io::Result<()> {
    print!("{}", registry_contents()?);
    Ok(())
}

fn registry_contents() -> io::Result<String> {
    match fs::read_to_string(registry_path()?) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(EMPTY_REGISTRY.to_string()),
        Err(error) => Err(error),
    }
}

fn refresh_registry() -> io::Result<PathBuf> {
    let apps = discover_apps()?;
    sync_desktop_entries(&apps)?;
    let json = registry_json(&apps);
    let path = registry_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, json)?;
    fs::rename(&temp_path, &path)?;

    Ok(path)
}

fn sync_desktop_entries(apps: &[App]) -> io::Result<()> {
    let data_home = data_home()?;
    let applications_dir = data_home.join("applications");
    let icons_dir = data_home.join("icons/hicolor/scalable/apps");
    fs::create_dir_all(&applications_dir)?;
    fs::create_dir_all(&icons_dir)?;

    let mut current_desktop_ids = Vec::new();

    for app in apps {
        current_desktop_ids.push(app.desktop_id.clone());
        write_desktop_entry(&applications_dir, app)?;
        write_fallback_icon(&icons_dir, app)?;
    }

    remove_stale_desktop_entries(&applications_dir, &current_desktop_ids)?;
    remove_stale_icons(&icons_dir, &current_desktop_ids)?;
    Ok(())
}

fn write_desktop_entry(applications_dir: &Path, app: &App) -> io::Result<()> {
    let path = applications_dir.join(format!("{}.desktop", app.desktop_id));
    let contents = format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec=droidianos-launch {}\nIcon={}\nCategories=Network;Utility;\nStartupNotify=true\nX-DroidianOS-Package={}\n",
        escape_desktop_value(&app.name),
        app.package,
        app.icon_name,
        app.package
    );

    write_file_if_changed(&path, &contents)
}

fn write_fallback_icon(icons_dir: &Path, app: &App) -> io::Result<()> {
    let path = icons_dir.join(format!("{}.svg", app.icon_name));
    let initials = app_initials(&app.name);
    let contents = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"128\" height=\"128\" viewBox=\"0 0 128 128\"><rect width=\"128\" height=\"128\" rx=\"24\" fill=\"#3b4252\"/><text x=\"64\" y=\"76\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"42\" font-weight=\"700\" fill=\"#ffffff\">{}</text></svg>\n",
        escape_xml(&initials)
    );

    write_file_if_changed(&path, &contents)
}

fn remove_stale_desktop_entries(applications_dir: &Path, current_desktop_ids: &[String]) -> io::Result<()> {
    let entries = match fs::read_dir(applications_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name.starts_with("droidianos-") || !file_name.ends_with(".desktop") {
            continue;
        }

        let desktop_id = file_name.trim_end_matches(".desktop");
        if !current_desktop_ids.iter().any(|current| current == desktop_id) {
            fs::remove_file(entry.path())?;
        }
    }

    Ok(())
}

fn remove_stale_icons(icons_dir: &Path, current_icon_names: &[String]) -> io::Result<()> {
    let entries = match fs::read_dir(icons_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name.starts_with("droidianos-") || !file_name.ends_with(".svg") {
            continue;
        }

        let icon_name = file_name.trim_end_matches(".svg");
        if !current_icon_names.iter().any(|current| current == icon_name) {
            fs::remove_file(entry.path())?;
        }
    }

    Ok(())
}

fn write_file_if_changed(path: &Path, contents: &str) -> io::Result<()> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == contents => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    fs::write(path, contents)
}

fn registry_path() -> io::Result<PathBuf> {
    Ok(data_home()?.join("droidianos/apps.json"))
}

fn data_home() -> io::Result<PathBuf> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(path) => PathBuf::from(path),
        None => {
            let home = env::var_os("HOME").ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "HOME is not set")
            })?;
            PathBuf::from(home).join(".local/share")
        }
    };

    Ok(data_home)
}

fn discover_apps() -> io::Result<Vec<App>> {
    let output = Command::new("waydroid")
        .args(["shell", "cmd", "package", "list", "packages", "-3"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("waydroid package list failed: {}", stderr.trim()),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut apps = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        let package = match line.strip_prefix("package:") {
            Some(package) => package.trim(),
            None => continue,
        };
        if package.is_empty() {
            continue;
        }
        let desktop_id = desktop_id(package);
        apps.push(App {
            package: package.to_string(),
            name: fallback_name(package),
            icon_name: desktop_id.clone(),
            desktop_id,
        });
    }

    apps.sort_by(|left, right| left.package.cmp(&right.package));
    apps.dedup_by(|left, right| left.package.as_str() == right.package.as_str());
    Ok(apps)
}

fn fallback_name(package: &str) -> String {
    package
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(package)
        .replace('_', " ")
}

fn desktop_id(package: &str) -> String {
    let mut id = String::from("droidianos-");

    for character in package.chars() {
        if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
            id.push(character);
        } else {
            id.push('-');
        }
    }

    id
}

fn app_initials(name: &str) -> String {
    let mut initials = String::new();

    for part in name.split_whitespace().take(2) {
        if let Some(character) = part.chars().next() {
            initials.push(character.to_ascii_uppercase());
        }
    }

    if initials.is_empty() {
        initials.push('A');
    }

    initials
}

fn registry_json(apps: &[App]) -> String {
    let mut json = String::from("{\"apps\":[");

    for (index, app) in apps.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"package\":\"");
        json.push_str(&escape_json(&app.package));
        json.push_str("\",\"name\":\"");
        json.push_str(&escape_json(&app.name));
        json.push_str("\",\"version\":null,\"activities\":[],\"icon\":\"");
        json.push_str(&escape_json(&app.icon_name));
        json.push_str("\",\"source\":\"apk\",\"permissions\":[]}");
    }

    json.push_str("]}\n");
    json
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

fn escape_desktop_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', " ").replace('\r', " ")
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::new();

    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            character => escaped.push(character),
        }
    }

    escaped
}
