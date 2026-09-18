#include <iostream>
#include <fstream>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>
#include <filesystem>

int main()
{
    const std::string detectionScript =
        "./scripts/detect-package-managers.sh";

    const char* outputEnvironment =
        std::getenv("LINUX_UPDATE_SCRIPT_OUTPUT");

    std::string updateScript;

    if (outputEnvironment &&
        outputEnvironment[0] != '\0')
    {
        updateScript = outputEnvironment;
    }
    else
    {
        updateScript =
            "./scripts/update-script-output.sh";
    }

    std::cout << "Linux Update Utility\n";
    std::cout << "====================\n\n";

    std::cout << "Detecting package managers...\n\n";

    FILE* pipe = popen(
        ("bash \"" + detectionScript + "\" 2>/dev/null").c_str(),
        "r"
    );

    if (!pipe)
    {
        std::cerr
            << "ERROR: Failed to run package manager "
            << "detection script.\n";

        return 1;
    }

    std::vector<std::string> packageManagers;

    char buffer[512];

    while (fgets(buffer, sizeof(buffer), pipe))
    {
        std::string line(buffer);

        while (!line.empty() &&
               (line.back() == '\n' ||
                line.back() == '\r'))
        {
            line.pop_back();
        }

        if (!line.empty())
        {
            packageManagers.push_back(line);
        }
    }

    pclose(pipe);

    if (packageManagers.empty())
    {
        std::cout
            << "No supported package managers detected.\n";

        return 0;
    }

    std::cout
        << "Detected package managers:\n";

    for (const auto& path : packageManagers)
    {
        std::string manager =
            std::filesystem::path(path)
                .filename()
                .string();

        std::cout
            << "  "
            << manager
            << "\n";
    }

    std::cout << "\n";

    std::filesystem::path outputPath(
        updateScript
    );

    std::filesystem::path outputDirectory =
        outputPath.parent_path();

    if (!outputDirectory.empty())
    {
        std::error_code error;

        std::filesystem::create_directories(
            outputDirectory,
            error
        );

        if (error)
        {
            std::cerr
                << "ERROR: Failed to create output directory: "
                << outputDirectory
                << "\n";

            return 1;
        }
    }

    std::ofstream script(
        updateScript
    );

    if (!script)
    {
        std::cerr
            << "ERROR: Failed to create "
            << updateScript
            << "\n";

        return 1;
    }

    script << "#!/bin/bash\n\n";
    script << "set -e\n\n";

    script << "echo \"Starting system update...\"\n\n";

    std::string privilegedCommands;
    bool needsPrivilege = false;
    bool hasFlatpak = false;

    for (const auto& path : packageManagers)
    {
        std::string manager =
            std::filesystem::path(path)
                .filename()
                .string();

        if (manager == "pacman")
        {
            needsPrivilege = true;

            privilegedCommands +=
                "echo \"Updating pacman packages...\"\n";

            privilegedCommands +=
                "pacman -Syu --noconfirm\n\n";
        }
        else if (manager == "apt")
        {
            needsPrivilege = true;

            privilegedCommands +=
                "echo \"Updating apt packages...\"\n";

            privilegedCommands +=
                "apt update && apt upgrade -y\n\n";
        }
        else if (manager == "dnf")
        {
            needsPrivilege = true;

            privilegedCommands +=
                "echo \"Updating dnf packages...\"\n";

            privilegedCommands +=
                "dnf upgrade -y\n\n";
        }
        else if (manager == "snap")
        {
            needsPrivilege = true;

            privilegedCommands +=
                "echo \"Updating Snap packages...\"\n";

            privilegedCommands +=
                "snap refresh\n\n";
        }
        else if (manager == "flatpak")
        {
            hasFlatpak = true;
        }
    }

    /*
     * All privileged package-manager operations use one
     * sudo invocation. The GUI supplies the password through
     * stdin using sudo -S.
     */
    if (needsPrivilege)
    {
        script
            << "sudo -S -p \"\" bash -c '"
            << privilegedCommands
            << "'\n\n";
    }

    /*
     * Flatpak remains a user-level operation.
     */
    if (hasFlatpak)
    {
        script
            << "echo \"Updating Flatpak packages...\"\n";

        script
            << "flatpak update -y\n\n";
    }

    script
        << "echo \"Update complete.\"\n";

    script.close();

    std::cout
        << "Generated update script:\n";

    std::cout
        << "========================\n\n";

    std::ifstream generated(
        updateScript
    );

    if (!generated)
    {
        std::cerr
            << "ERROR: Could not read generated "
            << "update script.\n";

        return 1;
    }

    std::string line;

    while (std::getline(
        generated,
        line
    ))
    {
        std::cout
            << line
            << '\n';
    }

    std::cout
        << "\n========================\n";

    std::cout
        << "Update script written to: "
        << updateScript
        << "\n";

    return 0;
}