//! Optional discovery only. Authentication and authorization remain sudo/PAM's job.
use std::{
    collections::HashSet,
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crate::backend::trusted_system_program;

fn pam_supports_fingerprint(directory: &Path, service: &str, seen: &mut HashSet<String>) -> bool {
    if seen.len() >= 32 || !seen.insert(service.to_owned()) {
        return false;
    }
    let Ok(contents) = fs::read_to_string(directory.join(service)) else {
        return false;
    };
    contents.lines().any(|line| {
        let line = line.split('#').next().unwrap_or("").trim();
        let mut fields = line.split_whitespace();
        let kind = fields.next().unwrap_or("").trim_start_matches('-');
        if kind == "@include" {
            return fields
                .next()
                .is_some_and(|name| pam_supports_fingerprint(directory, name, seen));
        }
        if kind != "auth" {
            return false;
        }
        let control = fields.next().unwrap_or("");
        if matches!(control, "include" | "substack") {
            return fields
                .next()
                .is_some_and(|name| pam_supports_fingerprint(directory, name, seen));
        }
        if control.starts_with('[') && !control.ends_with(']') {
            for field in fields.by_ref() {
                if field.ends_with(']') {
                    break;
                }
            }
        }
        fields.next().is_some_and(|module| {
            Path::new(module)
                .file_name()
                .is_some_and(|name| name == "pam_fprintd.so")
        })
    })
}

fn bus_call(path: &str, interface: &str, method: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(trusted_system_program("busctl")?)
        .args([
            "--system",
            "--timeout=2",
            "call",
            "net.reactivated.Fprint",
            path,
            interface,
            method,
        ])
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn configured() -> bool {
    if !pam_supports_fingerprint(Path::new("/etc/pam.d"), "sudo", &mut HashSet::new()) {
        return false;
    }
    let Some(devices) = bus_call(
        "/net/reactivated/Fprint/Manager",
        "net.reactivated.Fprint.Manager",
        "GetDevices",
        &[],
    ) else {
        return false;
    };
    devices
        .split('"')
        .skip(1)
        .step_by(2)
        .take(16)
        .any(|device| {
            // Empty username means the calling user, not an untrusted USER environment variable.
            bus_call(
                device,
                "net.reactivated.Fprint.Device",
                "ListEnrolledFingers",
                &["s", ""],
            )
            .is_some_and(|fingers| {
                fingers.split_whitespace().next() == Some("as")
                    && fingers
                        .split_whitespace()
                        .nth(1)
                        .and_then(|count| count.parse::<usize>().ok())
                        .is_some_and(|count| count > 0)
            })
        })
}

pub enum Event {
    Scanning,
    Finished(bool),
}

pub struct Attempt {
    pub receiver: mpsc::Receiver<Event>,
    cancel: Arc<AtomicBool>,
}
impl Drop for Attempt {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}
impl Attempt {
    #[cfg(test)]
    pub fn simulated() -> (Self, mpsc::Sender<Event>, Arc<AtomicBool>) {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        (
            Self {
                receiver,
                cancel: cancel.clone(),
            },
            sender,
            cancel,
        )
    }
    pub fn start() -> Self {
        let (sender, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancelled = cancel.clone();
        thread::spawn(move || {
            if !configured() || cancelled.load(Ordering::SeqCst) {
                let _ = sender.send(Event::Finished(false));
                return;
            }
            let _ = sender.send(Event::Scanning);
            let success = trusted_system_program("sudo").is_some_and(|sudo| {
                let mut command = Command::new(&sudo);
                command.args(["-S", "-p", "", "-v"]);
                if !validate(command, &cancelled, Duration::from_secs(35)) {
                    return false;
                }
                // Some policies disable timestamp caching. Keep their password
                // fallback in the dialog instead of starting an unusable operation.
                let mut cached = Command::new(sudo);
                cached.args(["-n", "-v"]);
                validate(cached, &cancelled, Duration::from_secs(2))
            });
            let _ = sender.send(Event::Finished(success));
        });
        Self { receiver, cancel }
    }
}

fn validate(mut command: Command, cancelled: &AtomicBool, timeout: Duration) -> bool {
    let Ok(mut child) = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    let started = Instant::now();
    loop {
        if cancelled.load(Ordering::SeqCst) || started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn follows_auth_includes_but_ignores_comments_sessions_and_cycles() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("sudo"), "# auth sufficient pam_fprintd.so\nsession include fingerprint\nauth include common-auth\n").unwrap();
        fs::write(
            dir.path().join("fingerprint"),
            "auth sufficient pam_fprintd.so\n",
        )
        .unwrap();
        fs::write(dir.path().join("common-auth"), "auth include sudo\n").unwrap();
        assert!(!pam_supports_fingerprint(
            dir.path(),
            "sudo",
            &mut HashSet::new()
        ));
        fs::write(
            dir.path().join("common-auth"),
            "auth substack fingerprint\n",
        )
        .unwrap();
        assert!(pam_supports_fingerprint(
            dir.path(),
            "sudo",
            &mut HashSet::new()
        ));
        fs::write(dir.path().join("sudo"), "@include fingerprint\n").unwrap();
        fs::write(
            dir.path().join("fingerprint"),
            "auth [success=done default=ignore] /lib/security/pam_fprintd.so\n",
        )
        .unwrap();
        assert!(pam_supports_fingerprint(
            dir.path(),
            "sudo",
            &mut HashSet::new()
        ));
    }
    #[test]
    fn validation_obeys_failure_cancellation_and_timeout() {
        for (script, cancelled, timeout, expected) in [
            ("exit 0", false, 1000, true),
            ("exit 1", false, 1000, false),
            ("exec sleep 10", true, 1000, false),
            ("exec sleep 10", false, 30, false),
        ] {
            let mut command = Command::new("bash");
            command.args(["-c", script]);
            assert_eq!(
                validate(
                    command,
                    &AtomicBool::new(cancelled),
                    Duration::from_millis(timeout)
                ),
                expected
            );
        }
    }
}
