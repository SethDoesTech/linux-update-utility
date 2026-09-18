cmake_minimum_required(VERSION 3.16)

project(
    LinuxUpdaterApp
    VERSION 1.0
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

find_package(
    Qt6 REQUIRED
    COMPONENTS Widgets
)

# Backend
add_executable(
    linux-update-utility
    ../src/backend/linux-update-utility.cpp
)

# GUI
add_executable(
    linux-updater-app
    main.cpp
    MainWindow.cpp
    MainWindow.hpp
    LegalNotices.cpp
    LegalNotices.hpp
    resources.qrc
)

# System-wide AppImage installer.
add_executable(
    luu-setup-utility
    setup-main.cpp
    SetupWindow.cpp
    SetupWindow.hpp
    LegalNotices.cpp
    LegalNotices.hpp
    setup-resources.qrc
)

# Conservative x86-64 baseline.
target_compile_options(
    linux-update-utility
    PRIVATE
    -march=x86-64
    -mtune=generic
)

target_compile_options(
    linux-updater-app
    PRIVATE
    -march=x86-64
    -mtune=generic
)

target_compile_options(
    luu-setup-utility
    PRIVATE
    -march=x86-64
    -mtune=generic
)

target_link_libraries(
    linux-updater-app
    PRIVATE
    Qt6::Widgets
)

target_link_libraries(
    luu-setup-utility
    PRIVATE
    Qt6::Widgets
)

# Keep the backend beside the GUI during development.
set_target_properties(
    linux-update-utility
    linux-updater-app
    luu-setup-utility
    PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY
    "${CMAKE_BINARY_DIR}"
)

#
# Installation / AppDir layout
#
# AppDir/
# ├── scripts/
# │   └── detect-package-managers.sh
# └── usr/
#     ├── bin/
#     │   ├── linux-updater-app
#     │   └── linux-update-utility
#     └── share/
#         ├── applications/
#         │   └── linux-update-utility.desktop
#         └── icons/
#             └── hicolor/
#                 └── scalable/
#                     └── apps/
#                         └── linux-update-utility.svg
#

install(
    TARGETS
        linux-updater-app
        linux-update-utility
    RUNTIME DESTINATION
        usr/bin
)

install(
    PROGRAMS
        ${CMAKE_CURRENT_SOURCE_DIR}/../scripts/detect-package-managers.sh
    DESTINATION
        scripts
)

install(
    FILES
        ${CMAKE_CURRENT_SOURCE_DIR}/luu-icon.svg
    DESTINATION
        usr/share/icons/hicolor/scalable/apps
    RENAME
        linux-update-utility.svg
)

install(
    FILES
        ${CMAKE_CURRENT_SOURCE_DIR}/linux-update-utility.desktop
    DESTINATION
        usr/share/applications
)
