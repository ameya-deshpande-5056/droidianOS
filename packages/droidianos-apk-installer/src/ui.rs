use std::env;
use std::io;
use std::process::Command;

use droidianos_dbus_lite::Connection;

const INSTALLER_SERVICE: &str = "org.droidianos.Installer";
const INSTALLER_INTERFACE: &str = "org.droidianos.Installer";
const OBJECT_PATH: &str = "/org/droidianos/Installer";

fn main() {
    if let Err(error) = run() {
        show_error("Install Application", &error.to_string());
        eprintln!("droidianos-apk-installer: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let path = env::args().nth(1).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "missing APK path")
    })?;
    let metadata = droidianos_apk::inspect_package(&path)?;
    let summary = metadata_summary(&path, &metadata);

    if !confirm_install(&summary)? {
        return Ok(());
    }

    let _ = Command::new("systemctl")
        .args(["--user", "start", "droidianos-apk-installerd.service"])
        .status();

    let connection = Connection::session()?;
    let transaction_id = connection.call_string_method_one_arg(
        INSTALLER_SERVICE,
        OBJECT_PATH,
        INSTALLER_INTERFACE,
        "InstallApk",
        &path,
        600_000,
    )?;

    show_info(
        "Install Application",
        &format!("Installation completed.\nTransaction: {}", transaction_id),
    );
    Ok(())
}

fn metadata_summary(path: &str, metadata: &droidianos_apk::ApkMetadata) -> String {
    let mut summary = String::new();
    summary.push_str("Install this application?\n\n");
    summary.push_str("File: ");
    summary.push_str(path);
    summary.push('\n');
    push_optional(&mut summary, "Name", metadata.app_label.as_deref());
    push_optional(&mut summary, "Package", metadata.package_name.as_deref());
    push_optional(&mut summary, "Version", metadata.version_name.as_deref());
    push_optional(&mut summary, "Version code", metadata.version_code.as_deref());
    summary.push_str("Size: ");
    summary.push_str(&format_size(metadata.apk_size_bytes));
    summary.push('\n');

    if metadata.permissions.is_empty() {
        summary.push_str("\nPermissions: none declared\n");
    } else {
        summary.push_str("\nPermissions:\n");
        for permission in metadata.permissions.iter().take(20) {
            summary.push_str("- ");
            summary.push_str(&permission.id);
            summary.push_str(" (");
            summary.push_str(&permission.class_name);
            summary.push_str(")\n");
        }
        if metadata.permissions.len() > 20 {
            summary.push_str("- ...\n");
        }
    }

    summary
}

fn push_optional(summary: &mut String, label: &str, value: Option<&str>) {
    if let Some(value) = value {
        summary.push_str(label);
        summary.push_str(": ");
        summary.push_str(value);
        summary.push('\n');
    }
}

fn format_size(bytes: u64) -> String {
    let mib = bytes as f64 / 1_048_576.0;
    format!("{:.1} MiB", mib)
}

fn confirm_install(text: &str) -> io::Result<bool> {
    let status = Command::new("zenity")
        .args([
            "--question",
            "--title",
            "Install Application",
            "--ok-label",
            "Install",
            "--cancel-label",
            "Cancel",
            "--width",
            "560",
            "--height",
            "420",
            "--text",
            text,
        ])
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
