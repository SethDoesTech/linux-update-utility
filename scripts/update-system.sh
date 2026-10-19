#!/bin/bash

set -e

# Use system locations only. The GUI supplies a sudo password through stdin;
# this script never stores it or accepts it as an argument.
PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
export PATH

echo "Starting system update..."
echo

privileged_commands="set -e
"
needs_privilege=false

if command -v pacman >/dev/null 2>&1; then
    needs_privilege=true
    privileged_commands+='echo "Updating pacman packages..."
pacman -Syu --noconfirm
'
fi

if command -v apt >/dev/null 2>&1; then
    needs_privilege=true
    privileged_commands+='echo "Updating apt packages..."
apt update
apt upgrade -y
'
fi

if command -v dnf >/dev/null 2>&1; then
    needs_privilege=true
    privileged_commands+='echo "Updating dnf packages..."
dnf upgrade -y
'
fi

if command -v snap >/dev/null 2>&1; then
    needs_privilege=true
    privileged_commands+='echo "Updating Snap packages..."
snap refresh
'
fi

if "$needs_privilege"; then
    sudo_path=$(command -v sudo)
    bash_path=$(command -v bash)
    "$sudo_path" -k -S -p "" -- "$bash_path" -c "$privileged_commands"
fi

if flatpak_path=$(command -v flatpak 2>/dev/null); then
    echo "Updating Flatpak packages..."
    "$flatpak_path" update -y
fi

echo "Update complete."
