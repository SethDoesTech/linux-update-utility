#!/usr/bin/env bash

# Build both release AppImages from the project root in one command.

set -euo pipefail

project_directory=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
app_directory="$project_directory/linux-updater-app"
build_directory="$app_directory/build"
input_directory="$project_directory/input-release"
release_directory="$app_directory/release"
linuxdeploy="$app_directory/linuxdeploy-x86_64.AppImage"
qt_plugin="$app_directory/linuxdeploy-plugin-qt-x86_64.AppImage"
linuxdeploy_url="https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
qt_plugin_url="https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-x86_64.AppImage"
updater_name="Linux_Update_Utility-x86_64.AppImage"
setup_name="LUU_Setup_Utility-x86_64.AppImage"

require_command()
{
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'ERROR: Required command was not found: %s\n' "$1" >&2
        exit 1
    fi
}

download_tool()
{
    local url=$1
    local destination=$2
    local temporary_download

    if [[ -e "$destination" ]]; then
        return
    fi

    temporary_download=$(mktemp "${TMPDIR:-/tmp}/luu-tool.XXXXXX")
    printf 'Downloading %s...\n' "$(basename "$destination")"

    if command -v curl >/dev/null 2>&1; then
        if ! curl --fail --location --retry 3 \
            --output "$temporary_download" "$url"; then
            rm -f "$temporary_download"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget --output-document="$temporary_download" "$url"; then
            rm -f "$temporary_download"
            exit 1
        fi
    else
        rm -f "$temporary_download"
        printf 'ERROR: curl or wget is required to download packaging tools.\n' >&2
        exit 1
    fi

    install -Dm755 "$temporary_download" "$destination"
    rm -f "$temporary_download"
}

require_command cmake
require_command qmake6
download_tool "$linuxdeploy_url" "$linuxdeploy"
download_tool "$qt_plugin_url" "$qt_plugin"

chmod +x "$linuxdeploy" "$qt_plugin"

# AppImage extraction avoids depending on a working FUSE mount. Arch-family
# systems also need stripping disabled for newer RELR-enabled libraries.
export APPIMAGE_EXTRACT_AND_RUN=${APPIMAGE_EXTRACT_AND_RUN:-1}

if [[ -r /etc/os-release ]]; then
    # shellcheck disable=SC1091
    source /etc/os-release
    distribution_family="${ID:-} ${ID_LIKE:-}"
    if [[ "$distribution_family" == *arch* ]]; then
        export NO_STRIP=${NO_STRIP:-1}
    fi
fi

temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/luu-release.XXXXXX")
temporary_updater_directory="$temporary_directory/updater"
temporary_setup_directory="$temporary_directory/setup"

cleanup()
{
    rm -rf "$temporary_directory"
}

trap cleanup EXIT

printf '\n[1/4] Configuring the project...\n'
cmake -S "$app_directory" -B "$build_directory"

printf '\n[2/4] Compiling the application...\n'
cmake --build "$build_directory" -j2

printf '\n[3/4] Building the updater AppImage...\n'
"$app_directory/package-appimage.sh" \
    "$build_directory" \
    "$temporary_updater_directory"

temporary_updater="$temporary_updater_directory/$updater_name"

printf '\n[4/4] Building the setup AppImage...\n'
"$app_directory/package-setup-utility.sh" \
    "$build_directory" \
    "$temporary_updater" \
    "$temporary_setup_directory"

mkdir -p "$input_directory" "$release_directory"

install -Dm755 \
    "$temporary_updater" \
    "$input_directory/$updater_name"

install -Dm755 \
    "$temporary_setup_directory/$setup_name" \
    "$release_directory/$setup_name"

printf '\nBuild complete.\n'
printf 'Updater: %s\n' "$input_directory/$updater_name"
printf 'Setup utility: %s\n' "$release_directory/$setup_name"
