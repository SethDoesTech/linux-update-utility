# Linux Update Utility

Linux Update Utility is a simple GUI for updating Linux systems.

It was created by Seth Langer and is licensed under GNU GPLv3. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

## Seth Langer

This is a bit about me, mostly written for myself. Feel free to skip this.

Hi! I'm Seth, also known as SethDoesTech and Hankk, and I made this goofy little app. I 
made it because I want it, and I made it in a way that would allow me to share it. This 
is the first complete app I've ever made, at the time of writing I'm just a highschool 
student who loves all things FOSS and Linux. I hope one day to get a degree in computer 
science and make Linux development my job. This is my first major step in that direction. 
While I've tried to make a Linux distro, taken Linux online courses and configure my own
Linux systems, this app is the first thing I have to show for my work. The first major 
thing I could potentially add to my resume. For clarity, no I didn't write the whole app 
myself. I meticulously guided Codex through it. However, this app is 100% mine as I made 
every decision along the way. The look, function, architecture, compatibility, etc. are 
all mine. I hope the fact I used AI as a tool doesn't turn you away. Thanks for reading.

## Design

The application is a front end for tools a Linux system already provides. It does not try to replace or reimplement package management.

 Qt GUI, small C++ backend, detects package-manager commands available on the host, generates a readable update script, runs that script and displays its output

The GUI handles interaction, output, and administrator authorization. The backend chooses explicit commands for detected package managers. Bash and the host operating system perform the actual updates.

Supported package managers are pacman, apt, dnf, snap and flatpak. The generated script
uses commands with confirmations disabled.

System package-manager commands use one sudo invocation. Flatpak runs as the normal user. Passwords are supplied through standard input and are not written into generated scripts or command-line arguments.I have done my best to make this app secure, passwords are never saved and sudo is used only in update scripts.

## Installation

The distributed LUU Setup Utility AppImage contains a tested updater AppImage. It can:

* install Linux Update Utility as a normal desktop application;
* create its launcher and icon;
* update an existing installation with the bundled updater AppImage.

The installed AppImage is stored in /usr/local/bin. The application continues to use the host's existing package managers and repositories.

## Source layout

| Path | Purpose |
| --- | --- |
| src/backend/linux-update-utility.cpp | Detects supported package managers and generates the update script. |
| scripts/detect-package-managers.sh | Minimal executable detection helper. |
| linux-updater-app/ | Qt updater GUI, setup utility, CMake configuration, resources, desktop files, and packaging scripts. |
| LICENSE | Complete GPLv3 text. |
| NOTICE | Copyright and project notice. |
| machine-readme | Detailed technical, maintainer, and release reference. |
| release-input/ | Local-only location for the tested updater AppImage embedded in a setup release. |

Generated build output, local AppImages, and downloaded packaging tools are excluded from public source control by .gitignore.

## Building

Note from Seth, I always try to build the app with versions of tools that are likely
to work on most systems. 

The project uses C++17, CMake, Qt 6 Widgets, and bash. AppImage packaging additionally uses linuxdeploy with its Qt plugin and standard AppImage inspection tools.

A normal development build is:

    cmake -S linux-updater-app -B linux-updater-app/build
    cmake --build linux-updater-app/build

This produces the updater GUI, backend, and setup utility in the build directory. See [machine-readme](machine-readme) for packaging prerequisites and release-maintenance details.

## Maintenance principles

* Keep package-manager commands explicit and easy to inspect.
* Keep the GUI separate from update-command generation.
* Avoid unnecessary dependencies and desktop-specific frameworks.
* Treat authentication, generated scripts, and installer paths as security-sensitive.
* Test package-manager or packaging changes before publishing a release.
* Do not claim support for systems that have not been tested.

## Legal

Both applications show a compact information dialog with the project copyright notice. The full GPLv3 text is provided in [LICENSE](LICENSE).
# linux-update-utility
