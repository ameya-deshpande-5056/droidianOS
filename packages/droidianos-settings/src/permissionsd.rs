use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use droidianos_dbus_lite::{Connection, Message};

const SERVICE_NAME: &str = "org.droidianos.Permissions";
const PERMISSIONS_INTERFACE: &str = "org.droidianos.Permissions";
const OBJECT_PATH: &str = "/org/droidianos/Permissions";
const LIST_PERMISSIONS: &str = "ListPermissions";
const SET_PERMISSION: &str = "SetPermission";
const PERMISSIONS_CHANGED: &str = "PermissionsChanged";

struct PermissionPolicy {
    package: String,
    permission: String,
    state: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("droidianos-permissionsd: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let connection = Connection::session_with_name(SERVICE_NAME)?;

    loop {
        if let Some(message) = connection.next_message(1000) {
            handle_message(&connection, &message);
        }
    }
}

fn handle_message(connection: &Connection, message: &Message) {
    if message.is_method(PERMISSIONS_INTERFACE, LIST_PERMISSIONS) {
        match message.string_arg().and_then(|package| list_permissions_json(&package)) {
            Ok(json) => connection.send_string_reply(message, &json),
            Err(error) => connection.send_error_reply(message, &error.to_string()),
        }
        return;
    }

    if message.is_method(PERMISSIONS_INTERFACE, SET_PERMISSION) {
        match message.string_triple_args().and_then(|(package, permission, state)| {
            set_permission(&package, &permission, &state)?;
            Ok(package)
        }) {
            Ok(package) => {
                connection.send_empty_reply(message);
                connection.send_string_signal(
                    OBJECT_PATH,
                    PERMISSIONS_INTERFACE,
                    PERMISSIONS_CHANGED,
                    &package,
                );
            }
            Err(error) => connection.send_error_reply(message, &error.to_string()),
        }
    }
}

fn list_permissions_json(package: &str) -> io::Result<String> {
    let policies = read_policies()?;
    let mut json = String::from("{\"package\":\"");
    json.push_str(&escape_json(package));
    json.push_str("\",\"permissions\":[");
    let mut first = true;

    for policy in policies.iter().filter(|policy| policy.package == package) {
        if !first {
            json.push(',');
        }
        first = false;
        json.push_str("{\"id\":\"");
        json.push_str(&escape_json(&policy.permission));
        json.push_str("\",\"state\":\"");
        json.push_str(&escape_json(&policy.state));
        json.push_str("\"}");
    }

    json.push_str("]}");
    Ok(json)
}

fn set_permission(package: &str, permission: &str, state: &str) -> io::Result<()> {
    validate_state(state)?;
    let mut policies = read_policies()?;
    policies.retain(|policy| !(policy.package == package && policy.permission == permission));
    policies.push(PermissionPolicy {
        package: package.to_string(),
        permission: permission.to_string(),
        state: state.to_string(),
    });
    write_policies(&policies)?;
    apply_android_permission(package, permission, state)
}

fn apply_android_permission(package: &str, permission: &str, state: &str) -> io::Result<()> {
    if !permission.starts_with("android.permission.") {
        return Ok(());
    }
    if state == "default" {
        return Ok(());
    }

    let command = if state == "allowed" { "grant" } else { "revoke" };
    let status = Command::new("waydroid")
        .args(["shell", "pm", command, package, permission])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("failed to {} {}", command, permission),
        ));
    }

    Ok(())
}

fn validate_state(state: &str) -> io::Result<()> {
    match state {
        "allowed" | "denied" | "default" => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "permission state must be allowed, denied, or default",
        )),
    }
}

fn read_policies() -> io::Result<Vec<PermissionPolicy>> {
    let path = policy_path()?;
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    Ok(parse_policies(&contents))
}

fn write_policies(policies: &[PermissionPolicy]) -> io::Result<()> {
    let path = policy_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut json = String::from("{\"permissions\":[");
    for (index, policy) in policies.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"package\":\"");
        json.push_str(&escape_json(&policy.package));
        json.push_str("\",\"permission\":\"");
        json.push_str(&escape_json(&policy.permission));
        json.push_str("\",\"state\":\"");
        json.push_str(&escape_json(&policy.state));
        json.push_str("\"}");
    }
    json.push_str("]}\n");

    fs::write(path, json)
}

fn parse_policies(contents: &str) -> Vec<PermissionPolicy> {
    let mut policies = Vec::new();
    let mut remainder = contents;

    while let Some(start) = remainder.find('{') {
        remainder = &remainder[start + 1..];
        let end = match remainder.find('}') {
            Some(end) => end,
            None => break,
        };
        let object = &remainder[..end];
        remainder = &remainder[end + 1..];

        let package = match json_string_field(object, "package") {
            Some(value) => value,
            None => continue,
        };
        let permission = match json_string_field(object, "permission") {
            Some(value) => value,
            None => continue,
        };
        let state = match json_string_field(object, "state") {
            Some(value) => value,
            None => continue,
        };
        policies.push(PermissionPolicy {
            package,
            permission,
            state,
        });
    }

    policies
}

fn policy_path() -> io::Result<PathBuf> {
    let config_home = match env::var_os("XDG_CONFIG_HOME") {
        Some(path) => PathBuf::from(path),
        None => {
            let home = env::var_os("HOME").ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "HOME is not set")
            })?;
            PathBuf::from(home).join(".config")
        }
    };

    Ok(config_home.join("droidianos/permissions.json"))
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
