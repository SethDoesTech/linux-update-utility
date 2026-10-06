#!/usr/bin/env bash

# Shared by both AppImage packagers after linuxdeploy's normal Qt pass.
# Qt loads these plugins dynamically, so scanning linked libraries misses them.
bundle_qt_platforms()
{
    local appdir=$1
    local qt_library_directory qt_plugin_directory plugin relative_path
    local -a wayland_platform_plugins plugin_paths library_arguments

    qt_library_directory=$("$qmake" -query QT_INSTALL_LIBS)
    qt_plugin_directory=$("$qmake" -query QT_INSTALL_PLUGINS)

    # Newer Qt 6 uses libqwayland.so; older Qt 6 releases use
    # libqwayland-generic.so and optionally libqwayland-egl.so.
    wayland_platform_plugins=("$qt_plugin_directory"/platforms/libqwayland*.so)
    require_file "${wayland_platform_plugins[0]}"
    plugin_paths=(
        "${wayland_platform_plugins[@]}"
        "$qt_plugin_directory/wayland-shell-integration/libxdg-shell.so"
    )
    # Include the Qt EGL integration when provided by the build's Qt runtime.
    plugin="$qt_plugin_directory/wayland-graphics-integration-client/libqt-plugin-wayland-egl.so"
    if [[ -f "$plugin" ]]; then
        plugin_paths+=("$plugin")
    fi

    library_arguments=(
        --library "$qt_library_directory/libQt6XcbQpa.so.6"
        --library "$qt_library_directory/libQt6WaylandClient.so.6"
    )
    require_file "$qt_library_directory/libQt6XcbQpa.so.6"
    require_file "$qt_library_directory/libQt6WaylandClient.so.6"

    for plugin in "${plugin_paths[@]}"; do
        require_file "$plugin"
        relative_path=${plugin#"$qt_plugin_directory/"}
        install -Dm755 "$plugin" "$appdir/usr/plugins/$relative_path"
        library_arguments+=(--deploy-deps-only "$appdir/usr/plugins/$relative_path")
    done

    # Scan the staged plugins too: linuxdeploy bundles their dependencies and
    # sets their RPATH to the AppDir's libraries. A plain copy is insufficient.
    "$linuxdeploy" --appdir "$appdir" "${library_arguments[@]}"

    require_file "$appdir/usr/plugins/platforms/libqxcb.so"
    require_file "$appdir/usr/lib/libQt6XcbQpa.so.6"
    require_file "$appdir/usr/lib/libQt6WaylandClient.so.6"
    for plugin in "${plugin_paths[@]}"; do
        relative_path=${plugin#"$qt_plugin_directory/"}
        require_file "$appdir/usr/plugins/$relative_path"
    done
}
