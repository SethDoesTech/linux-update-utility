# Rust desktop interfaces

The updater, setup utility, and package-manager backend are Rust. The backend
remains a separate executable so its generated script can be inspected directly.

From the repository root:

```sh
cmake -S linux-updater-app -B linux-updater-app/build
cmake --build linux-updater-app/build -j2
./linux-updater-app/build/linux-updater-app
./linux-updater-app/build/luu-setup-utility
```

Install Rust/Cargo 1.88+, CMake, pkg-config, and Linux X11,
Wayland, xkbcommon, and OpenGL development libraries. Cargo.lock is committed;
CMake uses `cargo build --locked --release`. Binaries keep their original names.
Use CMake for runnable builds: Cargo alone does not stage the backend and detection
script in the runtime layout expected by the GUI.

```sh
cargo fmt --manifest-path linux-updater-app/Cargo.toml --check
cargo clippy --locked --manifest-path linux-updater-app/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path linux-updater-app/Cargo.toml
```

`src/lib.rs` owns windows, dialogs, themes, and worker state. `src/backend.rs`
runs the included detector and atomically stages the included update script. `src/operations.rs`
owns subprocess execution, output streaming, system information, and setup/removal.
Both applications share masked password prompts. Operations run off the UI thread;
passwords go through stdin and owned password buffers are zeroed after use.

Theme settings retain the former Qt config path and theme names. Generated scripts
retain `$XDG_DATA_HOME/linux-update-utility/update-script-output.sh` (falling back
to `~/.local/share`). Package-manager commands remain in the included shell scripts.

AppImage scripts use linuxdeploy without the Qt plugin. See `../machine-readme`
for packaging and manual checks. Native Wayland/X11 display, accessibility, and
on-screen keyboard behavior must be tested on release target desktops.

Fingerprint authorization is attempted automatically when sudo's PAM auth stack
(including included files) enables `pam_fprintd.so` and fprintd reports an enrolled
finger for the current user on an attached reader. Discovery uses the optional
system `busctl` executable; missing tools, devices, enrollment, or configuration
leave the existing password dialog unchanged. No PAM or reader settings are changed.
The device and enrollment checks use the [fprintd D-Bus API](https://fprint.freedesktop.org/fprintd-dev/Device.html).

The password field stays focused during scanning. Typing, pasting, IME input, or
choosing **Use password** cancels the fingerprint attempt and keeps keyboard input
in the masked field. Cancel/closing also abandons the attempt. Failed or timed-out
scans leave password authorization available. Password submission still follows
the machine's PAM policy, including any fingerprint timeout that policy requires.

Fingerprint validation uses sudo/PAM, never a client-side biometric success flag
as permission to run privileged commands. Successful validation (or an existing
sudo credential) is reused with `sudo -n` from the same parent process; systems
that disable sudo credential caching fall back to password entry. Flatpak continues to
run as the original user. Hardware release checks should cover enrolled and
unenrolled readers, successful/failed scans, reader removal, typing/pasting/IME
during a scan, Cancel, and the password-only path on a machine without a reader.
