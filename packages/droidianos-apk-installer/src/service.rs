use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use droidianos_dbus_lite::{Connection, Message};

const SERVICE_NAME: &str = "org.droidianos.Installer";
const INSTALLER_INTERFACE: &str = "org.droidianos.Installer";
const OBJECT_PATH: &str = "/org/droidianos/Installer";
const INSPECT_APK: &str = "InspectApk";
const INSTALL_APK: &str = "InstallApk";
const GET_TRANSACTION: &str = "GetTransaction";
const INSTALL_PROGRESS: &str = "InstallProgress";

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
        eprintln!("droidianos-apk-installerd: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let connection = Connection::session_with_name(SERVICE_NAME)?;
    let mut transactions = HashMap::new();

    loop {
        if let Some(message) = connection.next_message(1000) {
            handle_message(&connection, &message, &mut transactions);
        }
    }
}

fn handle_message(
    connection: &Connection,
    message: &Message,
    transactions: &mut HashMap<String, Transaction>,
) {
    if message.is_method(INSTALLER_INTERFACE, INSPECT_APK) {
        match message.string_arg().and_then(|path| inspect_apk_json(&path)) {
            Ok(metadata) => connection.send_string_reply(message, &metadata),
            Err(error) => connection.send_error_reply(message, &error.to_string()),
        }
        return;
    }

    if message.is_method(INSTALLER_INTERFACE, INSTALL_APK) {
        match message.string_arg().and_then(|path| install_apk(connection, transactions, &path)) {
            Ok(transaction_id) => connection.send_string_reply(message, &transaction_id),
            Err(error) => connection.send_error_reply(message, &error.to_string()),
        }
        return;
    }

    if message.is_method(INSTALLER_INTERFACE, GET_TRANSACTION) {
        match message.string_arg() {
            Ok(transaction_id) => {
                let json = transactions
                    .get(&transaction_id)
                    .map(transaction_json)
                    .unwrap_or_else(|| missing_transaction_json(&transaction_id));
                connection.send_string_reply(message, &json);
            }
            Err(error) => connection.send_error_reply(message, &error.to_string()),
        }
    }
}

fn inspect_apk_json(path: &str) -> io::Result<String> {
    droidianos_apk::inspect_apk(path).map(|metadata| metadata.to_json())
}

fn install_apk(
    connection: &Connection,
    transactions: &mut HashMap<String, Transaction>,
    path: &str,
) -> io::Result<String> {
    inspect_apk_json(path)?;

    let id = transaction_id();
    set_transaction(connection, transactions, &id, "install", "running", 10, "Inspecting APK");
    let staged_apk = stage_apk(path, &id)?;

    set_transaction(connection, transactions, &id, "install", "running", 50, "Installing APK");
    let output = Command::new("waydroid")
        .args(["app", "install", staged_apk.to_string_lossy().as_ref()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        set_transaction(connection, transactions, &id, "install", "failed", 100, "Installation failed");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("waydroid app install failed: {}", stderr.trim()),
        ));
    }

    let _ = Command::new("droidianos-integrationd").arg("--refresh").status();
    set_transaction(connection, transactions, &id, "install", "complete", 100, "Installation complete");
    Ok(id)
}

fn stage_apk(path: &str, transaction_id: &str) -> io::Result<PathBuf> {
    let source = Path::new(path);
    let metadata = fs::metadata(source)?;
    if !metadata.is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "APK path is not a file"));
    }

    let staging_dir = cache_home()?.join("droidianos/apk-staging");
    fs::create_dir_all(&staging_dir)?;
    let target = staging_dir.join(format!("{}.apk", transaction_id));
    fs::copy(source, &target)?;
    Ok(target)
}

fn cache_home() -> io::Result<PathBuf> {
    let cache_home = match env::var_os("XDG_CACHE_HOME") {
        Some(path) => PathBuf::from(path),
        None => {
            let home = env::var_os("HOME").ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "HOME is not set")
            })?;
            PathBuf::from(home).join(".cache")
        }
    };

    Ok(cache_home)
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
    connection.send_progress_signal(
        OBJECT_PATH,
        INSTALLER_INTERFACE,
        INSTALL_PROGRESS,
        id,
        percent,
        message,
    );
}

fn transaction_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("apk-{}", millis)
}

fn transaction_json(transaction: &Transaction) -> String {
    format!(
        "{{\"id\":\"{}\",\"kind\":\"{}\",\"state\":\"{}\",\"percent\":{},\"message\":\"{}\"}}",
        droidianos_apk::escape_json(&transaction.id),
        droidianos_apk::escape_json(&transaction.kind),
        droidianos_apk::escape_json(&transaction.state),
        transaction.percent,
        droidianos_apk::escape_json(&transaction.message)
    )
}

fn missing_transaction_json(transaction_id: &str) -> String {
    format!(
        "{{\"id\":\"{}\",\"kind\":\"unknown\",\"state\":\"missing\",\"percent\":0,\"message\":\"Transaction not found\"}}",
        droidianos_apk::escape_json(transaction_id)
    )
}
