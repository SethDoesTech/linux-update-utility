use std::{
    env, fs,
    io::{self, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
};

const OUTPUT_VARIABLE: &str = "LINUX_UPDATE_SCRIPT_OUTPUT";
const DEFAULT_OUTPUT: &str = "./scripts/update-script-output.sh";
const SAFE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
const INCLUDED_UPDATE_SCRIPT: &str = include_str!("../../scripts/update-system.sh");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageManager {
    Pacman,
    Flatpak,
    Apt,
    Dnf,
    Snap,
}

impl PackageManager {
    pub fn executable(self) -> &'static str {
        match self {
            Self::Pacman => "pacman",
            Self::Flatpak => "flatpak",
            Self::Apt => "apt",
            Self::Dnf => "dnf",
            Self::Snap => "snap",
        }
    }

    fn from_path(path: &Path) -> Option<Self> {
        match path.file_name()?.to_str()? {
            "pacman" => Some(Self::Pacman),
            "flatpak" => Some(Self::Flatpak),
            "apt" => Some(Self::Apt),
            "dnf" => Some(Self::Dnf),
            "snap" => Some(Self::Snap),
            _ => None,
        }
    }

    fn is_privileged(self) -> bool {
        self != Self::Flatpak
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectedManager {
    pub manager: PackageManager,
    pub path: PathBuf,
}

pub fn detect_package_managers() -> Vec<DetectedManager> {
    detect_with_script(&detector_path()).unwrap_or_default()
}

fn detect_with_script(script: &Path) -> Result<Vec<DetectedManager>, String> {
    let bash = trusted_system_program("bash")
        .ok_or_else(|| "a trusted system Bash executable was not found".to_owned())?;
    let output = Command::new(bash)
        .arg(script)
        .env_clear()
        .env("PATH", SAFE_PATH)
        .env("LC_ALL", "C")
        .output()
        .map_err(|error| format!("could not run package-manager detector: {error}"))?;
    // The original handwritten detector is a sequence of `command -v` calls.
    // Its final status only reflects the last manager, so stdout is authoritative.
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            // Identify the command before resolving symlinks. On DNF5 systems,
            // /usr/bin/dnf resolves to /usr/bin/dnf5, but it is still the dnf
            // command that the detector found. Validate and retain the resolved
            // target so the trust checks apply to the executable that will run.
            let detected_path = Path::new(line.trim());
            let manager = PackageManager::from_path(detected_path)?;
            let path = fs::canonicalize(detected_path).ok()?;
            is_trusted_executable(&path).then_some(DetectedManager { manager, path })
        })
        .collect())
}

fn detector_path() -> PathBuf {
    application_root().join("scripts/detect-package-managers.sh")
}

fn application_root() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("../..")
}

pub(crate) fn trusted_system_program(name: &str) -> Option<PathBuf> {
    ["/usr/bin", "/usr/sbin", "/bin", "/sbin"]
        .into_iter()
        .map(|directory| absolute_path(&Path::new(directory).join(name)))
        .find(|path| is_trusted_executable(path))
}

fn is_trusted_executable(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    let Ok(parent_metadata) = fs::metadata(parent) else {
        return false;
    };
    metadata.is_file()
        && metadata.uid() == parent_metadata.uid()
        && metadata.permissions().mode() & 0o111 != 0
        && metadata.permissions().mode() & 0o022 == 0
        && parent_metadata.permissions().mode() & 0o022 == 0
}

fn absolute_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn current_update_script() -> Result<String, String> {
    if detect_package_managers().is_empty() {
        return Err("no supported package managers were detected".to_owned());
    }
    Ok(INCLUDED_UPDATE_SCRIPT.to_owned())
}

pub fn update_requires_privilege() -> bool {
    detect_package_managers()
        .iter()
        .any(|manager| manager.manager.is_privileged())
}

pub fn write_script(path: &Path, contents: &str) -> io::Result<()> {
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    if let Some(directory) = directory {
        fs::create_dir_all(directory)?;
    }
    let directory = directory.unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::Builder::new()
        .prefix(".update-script-")
        .tempfile_in(directory)?;
    temporary
        .as_file_mut()
        .set_permissions(fs::Permissions::from_mode(0o600))?;
    temporary.write_all(contents.as_bytes())?;
    temporary.as_file_mut().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

pub fn run() -> Result<(), String> {
    let output_path = env::var_os(OUTPUT_VARIABLE)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT));
    let managers = detect_with_script(&detector_path())?;

    println!("Linux Update Utility\n====================\n");
    println!("Detecting package managers...\n");
    if managers.is_empty() {
        println!("No supported package managers detected.");
        return Ok(());
    }
    println!("Detected package managers:");
    for manager in &managers {
        println!("  {}", manager.manager.executable());
    }

    write_script(&output_path, INCLUDED_UPDATE_SCRIPT)
        .map_err(|error| format!("could not write {}: {error}", output_path.display()))?;
    println!("\nUpdate script prepared successfully.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn included_update_script_has_one_privilege_boundary() {
        assert_eq!(INCLUDED_UPDATE_SCRIPT.matches(" -k -S ").count(), 1);
        assert!(INCLUDED_UPDATE_SCRIPT.contains("flatpak_path"));
        assert!(INCLUDED_UPDATE_SCRIPT.contains("PATH=/usr/local/sbin"));
    }

    #[test]
    fn detector_output_accepts_only_known_trusted_executables() {
        let managers = detect_package_managers();
        assert!(
            managers
                .iter()
                .all(|manager| is_trusted_executable(&manager.path))
        );
    }

    #[test]
    fn detector_recognizes_dnf_symlinked_to_dnf5() {
        let temp = tempfile::tempdir().unwrap();
        let dnf5 = temp.path().join("dnf5");
        fs::write(&dnf5, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&dnf5, fs::Permissions::from_mode(0o755)).unwrap();

        let dnf = temp.path().join("dnf");
        symlink(&dnf5, &dnf).unwrap();
        let detector = temp.path().join("detector.sh");
        fs::write(&detector, format!("printf '%s\\n' '{}'\n", dnf.display())).unwrap();

        let managers = detect_with_script(&detector).unwrap();
        assert_eq!(
            managers,
            vec![DetectedManager {
                manager: PackageManager::Dnf,
                path: fs::canonicalize(dnf5).unwrap(),
            }]
        );
        assert!(managers[0].manager.is_privileged());
    }

    #[test]
    fn atomically_writes_a_private_script() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("nested/update.sh");
        write_script(&path, "#!/bin/bash\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "#!/bin/bash\n");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
