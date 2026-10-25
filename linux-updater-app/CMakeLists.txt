cmake_minimum_required(VERSION 3.16)
project(LinuxUpdaterApp VERSION 1.2.1 LANGUAGES NONE)

find_program(CARGO_EXECUTABLE cargo)
if(NOT CARGO_EXECUTABLE)
    message(FATAL_ERROR "Rust/Cargo 1.88 or newer is required to build the GUI. Install Rust and make cargo available on PATH.")
endif()
set(RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo")
file(GLOB_RECURSE RUST_SOURCES CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/src/*.rs")
add_custom_command(
    OUTPUT "${CMAKE_BINARY_DIR}/linux-updater-app" "${CMAKE_BINARY_DIR}/luu-setup-utility" "${CMAKE_BINARY_DIR}/linux-update-utility"
    COMMAND "${CARGO_EXECUTABLE}" build --locked --release
        --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
        --target-dir "${RUST_TARGET_DIR}"
    COMMAND ${CMAKE_COMMAND} -E copy_if_different "${RUST_TARGET_DIR}/release/linux-updater-app" "${CMAKE_BINARY_DIR}/linux-updater-app"
    COMMAND ${CMAKE_COMMAND} -E copy_if_different "${RUST_TARGET_DIR}/release/luu-setup-utility" "${CMAKE_BINARY_DIR}/luu-setup-utility"
    COMMAND ${CMAKE_COMMAND} -E copy_if_different "${RUST_TARGET_DIR}/release/linux-update-utility" "${CMAKE_BINARY_DIR}/linux-update-utility"
    DEPENDS ${RUST_SOURCES} Cargo.toml Cargo.lock luu-icon.svg luu-icon.png
        ../scripts/detect-package-managers.sh ../scripts/update-system.sh
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    COMMENT "Building Rust updater and setup interfaces"
    VERBATIM
)
add_custom_target(rust-apps ALL DEPENDS "${CMAKE_BINARY_DIR}/linux-updater-app" "${CMAKE_BINARY_DIR}/luu-setup-utility" "${CMAKE_BINARY_DIR}/linux-update-utility")
install(PROGRAMS "${CMAKE_BINARY_DIR}/linux-updater-app" DESTINATION usr/bin)
install(PROGRAMS "${CMAKE_BINARY_DIR}/linux-update-utility" DESTINATION usr/bin)
install(PROGRAMS ../scripts/detect-package-managers.sh ../scripts/update-system.sh DESTINATION scripts)
install(FILES luu-icon.svg DESTINATION usr/share/icons/hicolor/scalable/apps RENAME linux-update-utility.svg)
install(FILES linux-update-utility.desktop DESTINATION usr/share/applications)
