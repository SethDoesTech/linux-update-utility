#!/usr/bin/env bash

# Build the Linux Update Utility AppImage on the supported Ubuntu release
# environment. This deliberately bundles only the Qt platform support needed
# for XCB and Wayland, rather than scanning unrelated desktop plugins.

set -euo pipefail

script_directory=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=package-qt-platforms.sh
source "$script_directory/package-qt-platforms.sh"
build_directory=${1:-"$script_directory/build"}
release_directory=${2:-"$script_directory/release"}

linuxdeploy=${LINUXDEPLOY:-"$script_directory/linuxdeploy-x86_64.AppImage"}
qt_plugin=${LINUXDEPLOY_PLUGIN_QT:-"$script_directory/linuxdeploy-plugin-qt-x86_64.AppImage"}
qmake=${QMAKE:-$(command -v qmake6 || true)}

app_id="linux-update-utility"
appimage_name="Linux_Update_Utility-x86_64.AppImage"
desktop_file="$script_directory/linux-update-utility.desktop"
icon_file="$script_directory/luu-icon.svg"
output_appimage="$release_directory/$appimage_name"

require_file()
{
    if [[ ! -e "$1" ]]; then
        printf 'ERROR: Required file was not found: %s\n' "$1" >&2
        exit 1
    fi
}

require_file "$build_directory/linux-updater-app"
require_file "$build_directory/linux-update-utility"
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
appdir=$(mktemp -d "${TMPDIR:-/tmp}/luu-appdir.XXXXXX")
package_directory=$(mktemp -d "${TMPDIR:-/tmp}/luu-package.XXXXXX")

cleanup()
{
    rm -rf "$appdir"
    rm -rf "$package_directory"
}

trap cleanup EXIT

cmake --install "$build_directory" --prefix "$appdir"

# linuxdeploy expects these AppImage desktop-integration files at AppDir root.
cp "$desktop_file" "$appdir/$app_id.desktop"
cp "$icon_file" "$appdir/$app_id.svg"
cp "$icon_file" "$appdir/.DirIcon"

chmod +x "$linuxdeploy" "$qt_plugin"

export QMAKE="$qmake"
export LINUXDEPLOY_PLUGIN_QT="$qt_plugin"

# Bundle the normal Qt Widgets runtime and the XCB platform plugin.
"$linuxdeploy" \
    --appdir "$appdir" \
    --desktop-file "$desktop_file" \
    --icon-file "$icon_file" \
    --plugin qt

bundle_qt_platforms "$appdir"

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
