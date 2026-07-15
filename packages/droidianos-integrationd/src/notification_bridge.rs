use std::collections::VecDeque;
use std::io;
use std::process::Command;
use std::thread;
use std::time::Duration;

const MAX_SEEN: usize = 200;

#[derive(Clone)]
struct Notification {
    key: String,
    title: String,
    body: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("droidianos-notification-bridge: {}", error);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut seen = VecDeque::new();

    loop {
        match read_notifications() {
            Ok(notifications) => {
                for notification in notifications {
                    if has_seen(&seen, &notification.key) {
                        continue;
                    }
                    remember(&mut seen, notification.key.clone());
                    send_linux_notification(&notification);
                }
            }
            Err(error) => {
                eprintln!("droidianos-notification-bridge: {}", error);
            }
        }
        thread::sleep(Duration::from_secs(5));
    }
}

fn read_notifications() -> io::Result<Vec<Notification>> {
    let output = Command::new("waydroid")
        .args(["shell", "cmd", "notification", "list"])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_notifications(&stdout))
}

fn parse_notifications(output: &str) -> Vec<Notification> {
    let mut notifications = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let package = first_value(line, &["pkg=", "package="]).unwrap_or("Android application");
        let title = first_value(line, &["title=", "tickerText=", "text="]).unwrap_or(package);
        let body = first_value(line, &["text=", "content=", "message="]).unwrap_or(line);
        let key = first_value(line, &["key=", "id="])
            .map(|value| value.to_string())
            .unwrap_or_else(|| line.to_string());

        notifications.push(Notification {
            key,
            title: title.to_string(),
            body: body.to_string(),
        });
    }

    notifications
}

fn first_value<'a>(line: &'a str, keys: &[&str]) -> Option<&'a str> {
    for key in keys {
        if let Some(value) = value_after(line, key) {
            return Some(value);
        }
    }
    None
}

fn value_after<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let start = line.find(key)? + key.len();
    let rest = &line[start..];
    if let Some(stripped) = rest.strip_prefix('"') {
        let end = stripped.find('"')?;
        return Some(&stripped[..end]);
    }
    let end = rest.find(' ').unwrap_or(rest.len());
    Some(&rest[..end])
}

fn send_linux_notification(notification: &Notification) {
    let _ = Command::new("notify-send")
        .args([
            "--app-name",
            "Applications",
            &notification.title,
            &notification.body,
        ])
        .status();
}

fn has_seen(seen: &VecDeque<String>, key: &str) -> bool {
    seen.iter().any(|value| value == key)
}

fn remember(seen: &mut VecDeque<String>, key: String) {
    seen.push_back(key);
    while seen.len() > MAX_SEEN {
        seen.pop_front();
    }
}
