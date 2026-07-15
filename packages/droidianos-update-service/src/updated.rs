use std::collections::HashMap;
use std::io;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use droidianos_dbus_lite::{Connection, Message};

const SERVICE_NAME: &str = "org.droidianos.Updates";
const UPDATES_INTERFACE: &str = "org.droidianos.Updates";
const OBJECT_PATH: &str = "/org/droidianos/Updates";
const CHECK_UPDATES: &str = "CheckUpdates";
const LIST_UPDATES: &str = "ListUpdates";
const APPLY_UPDATES: &str = "ApplyUpdates";
const UPDATES_CHANGED: &str = "UpdatesChanged";
const UPDATE_PROGRESS: &str = "UpdateProgress";

#[derive(Clone)]
struct UpdateItem {
    id: String,
    name: String,
    source: String,
    current_version: String,
    new_version: String,
}

#[derive(Clone)]
struct Transaction {
    id: String,
    kind: String,
    state: String,
    percent: u32,
    message: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("droidianos-updated: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let connection = Connection::session_with_name(SERVICE_NAME)?;
    let mut updates = Vec::new();
    let mut transactions = HashMap::new();

    loop {
        if let Some(message) = connection.next_message(1000) {
            handle_message(&connection, &message, &mut updates, &mut transactions);
        }
    }
}

fn handle_message(
    connection: &Connection,
    message: &Message,
    updates: &mut Vec<UpdateItem>,
    transactions: &mut HashMap<String, Transaction>,
) {
    if message.is_method(UPDATES_INTERFACE, CHECK_UPDATES) {
        let id = transaction_id("check");
        set_transaction(connection, transactions, &id, "check", "running", 10, "Checking updates");
        *updates = collect_updates();
        set_transaction(connection, transactions, &id, "check", "complete", 100, "Update check complete");
        connection.send_string_signal(OBJECT_PATH, UPDATES_INTERFACE, UPDATES_CHANGED, "");
        connection.send_string_reply(message, &id);
        return;
    }

    if message.is_method(UPDATES_INTERFACE, LIST_UPDATES) {
        connection.send_string_reply(message, &updates_json(updates));
        return;
    }

    if message.is_method(UPDATES_INTERFACE, APPLY_UPDATES) {
        let id = transaction_id("apply");
        set_transaction(connection, transactions, &id, "apply", "running", 5, "Applying updates");
        match apply_all_updates(connection, transactions, &id) {
            Ok(_) => {
                updates.clear();
                set_transaction(connection, transactions, &id, "apply", "complete", 100, "Updates applied");
                connection.send_string_signal(OBJECT_PATH, UPDATES_INTERFACE, UPDATES_CHANGED, "");
                connection.send_string_reply(message, &id);
            }
            Err(error) => {
                set_transaction(connection, transactions, &id, "apply", "failed", 100, "Update failed");
                connection.send_error_reply(message, &error.to_string());
            }
        }
    }
}

fn collect_updates() -> Vec<UpdateItem> {
    let mut updates = Vec::new();
    updates.extend(apt_updates());
    updates.extend(flatpak_updates());
    updates.extend(android_image_updates());
    updates
}

fn apt_updates() -> Vec<UpdateItem> {
    let output = Command::new("apt")
        .args(["list", "--upgradable"])
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut updates = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("Listing") || !line.contains('/') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let package_source = match parts.next() {
            Some(value) => value,
            None => continue,
        };
        let new_version = parts.next().unwrap_or("");
        let package = package_source.split('/').next().unwrap_or(package_source);
        updates.push(UpdateItem {
            id: format!("apt:{}", package),
            name: package.to_string(),
            source: "apt".to_string(),
            current_version: "".to_string(),
            new_version: new_version.to_string(),
        });
    }

    updates
}

fn flatpak_updates() -> Vec<UpdateItem> {
    let output = Command::new("flatpak")
        .args(["remote-ls", "--updates", "--columns=application,version"])
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut updates = Vec::new();

    for line in stdout.lines() {
        let mut parts = line.split('\t');
        let app_id = match parts.next() {
            Some(value) if !value.is_empty() => value,
            _ => continue,
        };
        let version = parts.next().unwrap_or("");
        updates.push(UpdateItem {
            id: format!("flatpak:{}", app_id),
            name: app_id.to_string(),
            source: "flatpak".to_string(),
            current_version: "".to_string(),
            new_version: version.to_string(),
        });
    }

    updates
}

fn android_image_updates() -> Vec<UpdateItem> {
    Vec::new()
}

fn apply_all_updates(
    connection: &Connection,
    transactions: &mut HashMap<String, Transaction>,
    id: &str,
) -> io::Result<()> {
    set_transaction(connection, transactions, id, "apply", "running", 20, "Updating system packages");
    let apt_status = Command::new("pkexec")
        .args(["apt-get", "upgrade", "-y"])
        .status()?;
    if !apt_status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "APT update failed"));
    }

    set_transaction(connection, transactions, id, "apply", "running", 60, "Updating Flatpak applications");
    let flatpak_status = Command::new("flatpak").args(["update", "-y"]).status();
    if let Ok(status) = flatpak_status {
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Flatpak update failed"));
        }
    }

    set_transaction(connection, transactions, id, "apply", "running", 85, "Updating application support image");
    let _ = Command::new("waydroid").arg("upgrade").status();
    Ok(())
}

fn set_transaction(
    connection: &Connection,
    transactions: &mut HashMap<String, Transaction>,
    id: &str,
    kind: &str,
    state: &str,
    percent: u32,
    message: &str,
) {
    transactions.insert(
        id.to_string(),
        Transaction {
            id: id.to_string(),
            kind: kind.to_string(),
            state: state.to_string(),
            percent,
            message: message.to_string(),
        },
    );
    connection.send_progress_signal(OBJECT_PATH, UPDATES_INTERFACE, UPDATE_PROGRESS, id, percent, message);
}

fn updates_json(updates: &[UpdateItem]) -> String {
    let mut json = String::from("{\"updates\":[");
    for (index, update) in updates.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"id\":\"");
        json.push_str(&escape_json(&update.id));
        json.push_str("\",\"name\":\"");
        json.push_str(&escape_json(&update.name));
        json.push_str("\",\"source\":\"");
        json.push_str(&escape_json(&update.source));
        json.push_str("\",\"current_version\":\"");
        json.push_str(&escape_json(&update.current_version));
        json.push_str("\",\"new_version\":\"");
        json.push_str(&escape_json(&update.new_version));
        json.push_str("\"}");
    }
    json.push_str("]}");
    json
}

fn transaction_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{}-{}", prefix, millis)
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
