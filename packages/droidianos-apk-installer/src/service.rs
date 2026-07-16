use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    droidianos_apk::inspect_package(path).map(|metadata| metadata.to_json())
}

fn install_apk(
    connection: &Connection,
    transactions: &mut HashMap<String, Transaction>,
    path: &str,
) -> io::Result<String> {
    let id = transaction_id();
    set_transaction(connection, transactions, &id, "install", "running", 10, "Inspecting APK");
    let staging_dir = waydroid_data_home()?.join("waydroid_tmp").join(&id);
    let prepared = match droidianos_apk::prepare_package(path, &staging_dir) {
        Ok(prepared) => prepared,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_dir);
            set_transaction(connection, transactions, &id, "install", "failed", 100, "Invalid application package");
            return Err(error);
        }
    };
    let package_name = prepared.metadata.package_name.as_deref().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "APK manifest has no package name")
    })?;

    set_transaction(connection, transactions, &id, "install", "running", 30, "Starting Android");
    if let Err(error) = ensure_waydroid_session() {
        let _ = fs::remove_dir_all(&staging_dir);
        set_transaction(connection, transactions, &id, "install", "failed", 100, "Android did not start");
        return Err(error);
    }

    set_transaction(connection, transactions, &id, "install", "running", 50, "Installing APK");
    let install_result = install_prepared_package(&prepared.apk_files, &id);
    let _ = fs::remove_dir_all(&staging_dir);
    let output = match install_result {
        Ok(output) => output,
        Err(error) => {
            set_transaction(connection, transactions, &id, "install", "failed", 100, "Installation failed");
            return Err(error);
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        set_transaction(connection, transactions, &id, "install", "failed", 100, "Installation failed");
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Android package installation failed: {} {}", stdout.trim(), stderr.trim()),
        ));
    }

    if let Err(error) = verify_installed_package(package_name) {
        set_transaction(connection, transactions, &id, "install", "failed", 100, "Installation could not be verified");
        return Err(error);
    }

    let _ = Command::new("droidianos-integrationd").arg("--refresh").status();
    set_transaction(connection, transactions, &id, "install", "complete", 100, "Installation complete");
    Ok(id)
}

fn install_prepared_package(apk_files: &[PathBuf], transaction_id: &str) -> io::Result<Output> {
    if apk_files.len() == 1 {
        return Command::new("waydroid")
            .args(["app", "install"])
            .arg(&apk_files[0])
            .output();
    }

    let mut command = Command::new("pkexec");
    command
        .arg("/usr/lib/droidianos/install-split-apks")
        .arg(transaction_id);
    for apk_file in apk_files {
        command.arg(apk_file);
    }
    command.output()
}

fn ensure_waydroid_session() -> io::Result<()> {
    if waydroid_session_running() {
        return Ok(());
    }

    Command::new("waydroid")
        .args(["session", "start"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    for _ in 0..120 {
        thread::sleep(Duration::from_secs(1));
        if waydroid_session_running() {
            return Ok(());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "Waydroid session did not become ready; a working Wayland session is required",
    ))
}

fn waydroid_session_running() -> bool {
    let output = match Command::new("waydroid").arg("status").output() {
        Ok(output) => output,
        Err(_) => return false,
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    output.status.success()
        && stdout.lines().any(|line| {
            let mut fields = line.split_whitespace();
            fields.next() == Some("Session:")
                && fields.next() == Some("RUNNING")
                && fields.next().is_none()
        })
}

fn verify_installed_package(package_name: &str) -> io::Result<()> {
    let output = Command::new("waydroid")
        .args(["app", "list"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success()
        && stdout.lines().any(|line| {
            line.trim()
                .strip_prefix("packageName:")
                .map(str::trim)
                == Some(package_name)
        })
    {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("APK installation could not be verified: {}", stderr.trim()),
    ))
}

fn waydroid_data_home() -> io::Result<PathBuf> {
    let data_home = match env::var_os("XDG_DATA_HOME") {
        Some(path) => PathBuf::from(path),
        None => {
            let home = env::var_os("HOME").ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "HOME is not set")
            })?;
            PathBuf::from(home).join(".local/share")
        }
    };

    Ok(data_home.join("waydroid/data"))
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
