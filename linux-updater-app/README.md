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
