use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::Sender,
    thread,
};
use zeroize::{Zeroize, Zeroizing};

use crate::backend;

pub const INSTALLED: &str = "/usr/local/bin/linux-update-utility.AppImage";
const DESKTOP: &str = "/usr/share/applications/linux-update-utility.desktop";
const ICON: &str = "/usr/share/icons/hicolor/scalable/apps/linux-update-utility.svg";
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, PartialEq)]
pub enum Action {
    Generate,
    PrepareUpdate,
    Update,
    Install,
    Uninstall,
    Info,
}
pub enum Event {
    Output(String),
    Done(Action, Result<String>),
    PasswordRequired,
}

pub fn xdg_path(variable: &str, fallback: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(fallback))
}
pub fn script_path() -> PathBuf {
    xdg_path("XDG_DATA_HOME", ".local/share").join("linux-update-utility/update-script-output.sh")
}
fn executable_directory() -> Result<PathBuf> {
    env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Cannot locate executable directory".into())
}
fn application_root() -> Result<PathBuf> {
    Ok(executable_directory()?.join("../.."))
}
fn generate() -> Result<String> {
    generate_with(
        &executable_directory()?.join("linux-update-utility"),
        &application_root()?,
        &script_path(),
    )
}

fn generate_with(backend: &Path, root: &Path, path: &Path) -> Result<String> {
    let parent = path.parent().ok_or("Invalid script output path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    // The backend can exit successfully without generating anything when no
    // managers are detected. Never mistake a previous script for a fresh one.
    let staging = tempfile::tempdir_in(parent).map_err(|e| e.to_string())?;
    let generated = staging.path().join("update.sh");
    let output = Command::new(backend)
        .current_dir(root)
        .env("LINUX_UPDATE_SCRIPT_OUTPUT", &generated)
        .output()
        .map_err(|e| format!("Cannot start backend: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "Backend failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    promote_generated_script(&generated, path)?;
    Ok("The update script was generated successfully.".into())
}

fn promote_generated_script(generated: &Path, destination: &Path) -> Result<()> {
    if !generated.is_file() {
        return Err("Backend did not produce an update script. No supported package managers may be available.".into());
    }
    fs::rename(generated, destination).map_err(|e| format!("Cannot save generated script: {e}"))?;
    Ok(())
}

// Stream both pipes on separate readers so a full stderr pipe cannot deadlock stdout.
fn stream(mut command: Command, password: Zeroizing<String>, tx: &Sender<Event>) -> Result<()> {
    let mut password = password;
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Cannot start process: {e}"))?;
    let readers: Vec<Box<dyn Read + Send>> = vec![
        Box::new(child.stdout.take().unwrap()),
        Box::new(child.stderr.take().unwrap()),
    ];
    let handles: Vec<_> = readers
        .into_iter()
        .map(|mut reader| {
            let tx = tx.clone();
            thread::spawn(move || {
                let mut buffer = [0; 4096];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let _ = tx.send(Event::Output(
                                String::from_utf8_lossy(&buffer[..n]).into_owned(),
                            ));
                        }
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(e) => {
                            let _ = tx.send(Event::Output(format!("\nOutput read failed: {e}\n")));
                            break;
                        }
                    }
                }
            })
        })
        .collect();
    let input_result = if let Some(mut input) = child.stdin.take() {
        if password.is_empty() {
            Ok(())
        } else {
            input
                .write_all(password.as_bytes())
                .and_then(|_| input.write_all(b"\n"))
        }
    } else {
        Ok(())
    };
    password.zeroize();
    if let Err(error) = input_result {
        let _ = child.kill();
        let _ = child.wait();
        for handle in handles {
            let _ = handle.join();
        }
        return Err(format!("Cannot send authorization: {error}"));
    }
    let status = child.wait().map_err(|e| e.to_string());
    for handle in handles {
        let _ = handle.join();
    }
    let status = status?;
    if !status.success() {
        return Err(format!(
            "Process failed ({status}). Check the output above."
        ));
    }
    Ok(())
}
fn trusted_program(name: &str) -> Result<PathBuf> {
    backend::trusted_system_program(name)
        .ok_or_else(|| format!("Required system program was not found: {name}"))
}
fn command(name: &str) -> Result<Command> {
    Ok(Command::new(trusted_program(name)?))
}
fn sudo(script: &str) -> Result<Command> {
    let shell = trusted_program("bash")?;
    let mut cmd = command("sudo")?;
    cmd.args(["-k", "-S", "-p", "", "--"])
        .arg(shell)
        .args(["-c", script, "luu-gui"]);
    Ok(cmd)
}
fn install(password: Zeroizing<String>, tx: &Sender<Event>) -> Result<String> {
    let directory = executable_directory()?;
    let embedded = directory
        .join("../share/linux-update-utility/payload/Linux_Update_Utility-x86_64.AppImage");
    let source = if embedded.is_file() {
        embedded
    } else {
        directory.join("Linux_Update_Utility-x86_64.AppImage")
    };
    // Root cannot always read a user's AppImage FUSE mount. Stage in private temporary files.
    let stage = tempfile::tempdir().map_err(|e| e.to_string())?;
    let payload = stage.path().join("payload.AppImage");
    let icon = stage.path().join("icon.svg");
    let desktop = stage.path().join("launcher.desktop");
    fs::copy(source, &payload).map_err(|e| format!("Cannot stage updater payload: {e}"))?;
    fs::write(&icon, include_bytes!("../luu-icon.svg")).map_err(|e| e.to_string())?;
    fs::write(&desktop, format!("[Desktop Entry]\nType=Application\nName=Linux Update Utility\nComment=Update your Linux system through a graphical interface\nExec={INSTALLED}\nIcon={ICON}\nTerminal=false\nCategories=System;Settings;\nStartupNotify=true\nStartupWMClass=linux-update-utility\n")).map_err(|e| e.to_string())?;
    let mut cmd = sudo("set -e\ninstall -Dm755 \"$1\" \"${2}.new\"\nmv -f \"${2}.new\" \"$2\"\ninstall -Dm644 \"$3\" \"$4\"\ninstall -Dm644 \"$5\" \"$6\"\n")?;
    cmd.arg(payload)
        .arg(INSTALLED)
        .arg(icon)
        .arg(ICON)
        .arg(desktop)
        .arg(DESKTOP);
    stream(cmd, password, tx)?;
    match Command::new(INSTALLED).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
        Ok(mut child) => { thread::spawn(move || { let _ = child.wait(); }); Ok("Setup complete. Linux Update Utility is available from your applications menu.".into()) }
        Err(e) => Ok(format!("Setup complete, but automatic launch failed: {e}. Open Linux Update Utility from your applications menu.")),
    }
}
fn system_info() -> Result<String> {
    let distribution = fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .lines()
        .find_map(|l| {
            l.strip_prefix("PRETTY_NAME=")
                .map(|s| s.trim_matches('"').to_owned())
        })
        .unwrap_or_else(|| "Unknown".into());
    let kernel =
        fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_else(|_| "Unknown".into());
    let managers = backend::detect_package_managers()
        .iter()
        .map(|manager| manager.manager.executable())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!("Distribution: {distribution}\nArchitecture: {}\nKernel: {}\nPackage Managers: {}\nUpdate Script: {}", env::consts::ARCH, kernel.trim(), if managers.is_empty() { "None detected" } else { &managers }, if script_path().exists() { "Generated" } else { "Not generated" }))
}
pub fn perform(action: Action, password: Zeroizing<String>, tx: Sender<Event>) {
    let result = match action {
        Action::Generate => generate(),
        Action::PrepareUpdate => generate().and_then(|_| {
            if backend::update_requires_privilege() {
                let _ = tx.send(Event::PasswordRequired);
                Ok(String::new())
            } else {
                let mut cmd = command("bash")?;
                let script = backend::current_update_script()?;
                cmd.args(["-c", &script]).current_dir(application_root()?);
                stream(cmd, password, &tx).map(|_| "Update completed successfully.".into())
            }
        }),
        Action::Update => application_root().and_then(|root| {
            let mut cmd = command("bash")?;
            let script = backend::current_update_script()?;
            cmd.args(["-c", &script]).current_dir(root);
            stream(cmd, password, &tx).map(|_| "Update completed successfully.".into())
        }),
        Action::Install => install(password, &tx),
        Action::Uninstall => sudo("set -e\nrm -f -- \"$1\" \"$2\" \"$3\"\n").and_then(|mut cmd| {
            cmd.args([INSTALLED, DESKTOP, ICON]);
            stream(cmd, password, &tx).map(|_| {
                "Linux Update Utility was removed. Your user-local data was preserved.".into()
            })
        }),
        Action::Info => system_info(),
    };
    let _ = tx.send(Event::Done(action, result));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    #[test]
    fn generation_promotion_rejects_stale_scripts() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("data/update.sh");
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(&output, "# old script\n").unwrap();
        let missing = temp.path().join("missing.sh");
        assert!(promote_generated_script(&missing, &output).is_err());
        assert_eq!(fs::read_to_string(&output).unwrap(), "# old script\n");
        let generated = temp.path().join("generated.sh");
        fs::write(&generated, "# new script\n").unwrap();
        promote_generated_script(&generated, &output).unwrap();
        assert_eq!(fs::read_to_string(&output).unwrap(), "# new script\n");
    }

    #[test]
    fn streams_both_pipes_and_stdin_without_shell_interpolation() {
        let (tx, rx) = mpsc::channel();
        let mut cmd = Command::new("bash");
        cmd.args(["-c", "IFS= read -r value; test \"$value\" = 'literal $(false)'; result=$?; printf stdout; printf stderr >&2; exit $result"]);
        stream(cmd, Zeroizing::new("literal $(false)".into()), &tx).unwrap();
        drop(tx);
        let output: String = rx
            .into_iter()
            .filter_map(|e| {
                if let Event::Output(s) = e {
                    Some(s)
                } else {
                    None
                }
            })
            .collect();
        assert!(output.contains("stdout") && output.contains("stderr"));
        assert!(!output.contains("literal $(false)"));
    }
    #[test]
    fn process_failure_is_reported() {
        let (tx, _) = mpsc::channel();
        let mut cmd = Command::new("bash");
        cmd.args(["-c", "exit 7"]);
        assert!(stream(cmd, Zeroizing::new(String::new()), &tx)
            .unwrap_err()
            .contains('7'));
    }
}
