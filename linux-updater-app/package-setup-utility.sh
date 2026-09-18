#!/usr/bin/env bash

# Package the setup utility separately from the updater AppImage. The payload
# is copied unchanged into the setup AppImage; it is not rebuilt or scanned.

set -euo pipefail

script_directory=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
project_directory=$(cd "$script_directory/.." && pwd)
build_directory=${1:-"$script_directory/build"}
payload_path=${2:-"$project_directory/release-input/Linux_Update_Utility-x86_64.AppImage"}
release_directory=${3:-"$script_directory/release"}

linuxdeploy=${LINUXDEPLOY:-"$script_directory/linuxdeploy-x86_64.AppImage"}
qt_plugin=${LINUXDEPLOY_PLUGIN_QT:-"$script_directory/linuxdeploy-plugin-qt-x86_64.AppImage"}
qmake=${QMAKE:-$(command -v qmake6 || true)}

setup_name="LUU_Setup_Utility-x86_64.AppImage"
payload_name="Linux_Update_Utility-x86_64.AppImage"
output_appimage="$release_directory/$setup_name"
desktop_file="$script_directory/luu-setup-utility.desktop"
icon_file="$script_directory/luu-icon.svg"

require_file()
{
    if [[ ! -e "$1" ]]; then
        printf 'ERROR: Required file was not found: %s\n' "$1" >&2
        exit 1
    fi
}

require_file "$build_directory/luu-setup-utility"
require_file "$payload_path"
require_file "$linuxdeploy"
require_file "$qt_plugin"
require_file "$desktop_file"
require_file "$icon_file"

if [[ -z "$qmake" ]]; then
    printf 'ERROR: qmake6 was not found. Install the Qt 6 development tools.\n' >&2
    exit 1
fi

if [[ -e "$output_appimage" ]]; then
    printf 'ERROR: Refusing to overwrite existing artifact: %s\n' \
        "$output_appimage" >&2
    exit 1
fi

mkdir -p "$release_directory"
appdir=$(mktemp -d "${TMPDIR:-/tmp}/luu-setup-appdir.XXXXXX")
package_directory=$(mktemp -d "${TMPDIR:-/tmp}/luu-setup-package.XXXXXX")

cleanup()
{
    rm -rf "$appdir"
    rm -rf "$package_directory"
}

trap cleanup EXIT

install -Dm755 \
    "$build_directory/luu-setup-utility" \
    "$appdir/usr/bin/luu-setup-utility"

install -Dm644 \
    "$payload_path" \
    "$appdir/usr/share/linux-update-utility/payload/$payload_name"

chmod +x "$linuxdeploy" "$qt_plugin"

export QMAKE="$qmake"
export LINUXDEPLOY_PLUGIN_QT="$qt_plugin"

"$linuxdeploy" \
    --appdir "$appdir" \
    --desktop-file "$desktop_file" \
    --icon-file "$icon_file" \
    --plugin qt

embedded_payload="$appdir/usr/share/linux-update-utility/payload/$payload_name"
require_file "$embedded_payload"

(
    cd "$package_directory"
    ARCH=x86_64 "$linuxdeploy" \
        --appdir "$appdir" \
        --output appimage
)

generated_appimage=$(find "$package_directory" -maxdepth 1 -type f \
    -name '*.AppImage' -printf '%f\n' | sort | tail -n 1)

if [[ -z "$generated_appimage" ]]; then
    printf 'ERROR: linuxdeploy did not create an AppImage.\n' >&2
    exit 1
fi

mv "$package_directory/$generated_appimage" "$output_appimage"

printf 'Created: %s\n' "$output_appimage"
