#!/usr/bin/env bash

# winit loads these libraries at runtime, so ELF dependency scanning alone
# misses them. Keep OpenGL/EGL vendor drivers on the host.
rust_runtime_arguments()
{
    local library path
    RUST_RUNTIME_ARGS=()
    for library in libX11.so.6 libXcursor.so.1 libXi.so.6 libXrandr.so.2 \
        libxkbcommon.so.0 libwayland-client.so.0 libwayland-cursor.so.0 \
        libwayland-egl.so.1; do
        path=$(ldconfig -p | awk -v name="$library" '$1 == name && /x86-64/ && !found++ {print $NF}')
        if [[ -z "$path" || ! -f "$path" ]]; then
            printf 'ERROR: Required GUI runtime library was not found: %s\n' "$library" >&2
            return 1
        fi
        RUST_RUNTIME_ARGS+=(--library "$path")
    done
}
