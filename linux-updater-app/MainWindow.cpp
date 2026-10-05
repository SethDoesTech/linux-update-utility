#include "MainWindow.hpp"
#include "LegalNotices.hpp"
#include "PasswordDialog.hpp"

#include <QPushButton>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QWidget>
#include <QProcess>
#include <QMessageBox>
#include <QCoreApplication>
#include <QFile>
#include <QDir>
#include <QDialog>
#include <QFormLayout>
#include <QLabel>
#include <QSysInfo>
#include <QFileInfo>
#include <QPlainTextEdit>
#include <QFont>
#include <QTextCursor>
#include <QStandardPaths>
#include <QProcessEnvironment>
#include <QAction>
#include <QActionGroup>
#include <QApplication>
#include <QColor>
#include <QMenu>
#include <QMenuBar>
#include <QPalette>
#include <QSettings>

namespace
{
const QString kInstalledAppImage =
    "/usr/local/bin/linux-update-utility.AppImage";

const QString kDesktopFile =
    "/usr/share/applications/linux-update-utility.desktop";

const QString kIconFile =
    "/usr/share/icons/hicolor/scalable/apps/linux-update-utility.svg";
}

MainWindow::MainWindow()
    : uninstallAction(nullptr),
      lightThemeAction(nullptr),
      darkThemeAction(nullptr),
      greenThemeAction(nullptr),
      amberThemeAction(nullptr),
      blueThemeAction(nullptr),
      creditLabel(nullptr),
      process(nullptr)
{
    setupUI();

    connect(
        generateButton,
        &QPushButton::clicked,
        this,
        &MainWindow::generateScript
    );

    connect(
        updateButton,
        &QPushButton::clicked,
        this,
        &MainWindow::updateSystem
    );

    connect(
        systemInfoButton,
        &QPushButton::clicked,
        this,
        &MainWindow::showSystemInfo
    );

    connect(
        uninstallAction,
        &QAction::triggered,
        this,
        &MainWindow::uninstallApplication
    );
}

QString MainWindow::applicationRoot() const
{
    return QDir(
        QCoreApplication::applicationDirPath()
    ).absoluteFilePath("../..");
}

QString MainWindow::updateScriptPath() const
{
    QString dataDirectory =
        QStandardPaths::writableLocation(
            QStandardPaths::AppLocalDataLocation
        );

    return QDir(dataDirectory).filePath(
        "update-script-output.sh"
    );
}

void MainWindow::setupUI()
{
    setWindowTitle("Linux Update Utility");
    resize(600, 400);

    QMenu *settingsMenu =
        menuBar()->addMenu("Settings");

    QMenu *themeMenu =
        settingsMenu->addMenu("Theme");

    QActionGroup *themeGroup =
        new QActionGroup(this);

    themeGroup->setExclusive(true);

    lightThemeAction =
        themeMenu->addAction("Light");

    darkThemeAction =
        themeMenu->addAction("Dark");

    themeMenu->addSeparator();

    greenThemeAction =
        themeMenu->addAction("Green Phosphor");

    amberThemeAction =
        themeMenu->addAction("Amber CRT");

    blueThemeAction =
        themeMenu->addAction("Classic Blue");

    lightThemeAction->setCheckable(true);
    darkThemeAction->setCheckable(true);
    greenThemeAction->setCheckable(true);
    amberThemeAction->setCheckable(true);
    blueThemeAction->setCheckable(true);
    themeGroup->addAction(lightThemeAction);
    themeGroup->addAction(darkThemeAction);
    themeGroup->addAction(greenThemeAction);
    themeGroup->addAction(amberThemeAction);
    themeGroup->addAction(blueThemeAction);

    connect(
        lightThemeAction,
        &QAction::triggered,
        this,
        [this]()
        {
            applyTheme("light");
        }
    );

    connect(
        darkThemeAction,
        &QAction::triggered,
        this,
        [this]()
        {
            applyTheme("dark");
        }
    );

    connect(
        greenThemeAction,
        &QAction::triggered,
        this,
        [this]()
        {
            applyTheme("green-phosphor");
        }
    );

    connect(
        amberThemeAction,
        &QAction::triggered,
        this,
        [this]()
        {
            applyTheme("amber-crt");
        }
    );

    connect(
        blueThemeAction,
        &QAction::triggered,
        this,
        [this]()
        {
            applyTheme("classic-blue");
        }
    );

    settingsMenu->addSeparator();

    uninstallAction =
        settingsMenu->addAction("Uninstall Linux Update Utility...");

    QWidget *centralWidget =
        new QWidget(this);

    setCentralWidget(
        centralWidget
    );

    QVBoxLayout *layout =
        new QVBoxLayout(
            centralWidget
        );

    generateButton =
        new QPushButton(
            "Generate Script",
            this
        );

    updateButton =
        new QPushButton(
            "Update System",
            this
        );

    systemInfoButton =
        new QPushButton(
            "System Info",
            this
        );

    generateButton->setMinimumHeight(50);
    updateButton->setMinimumHeight(50);

    layout->addSpacing(40);

    layout->addWidget(
        generateButton
    );

    layout->addSpacing(15);

    layout->addWidget(
        updateButton
    );

    terminalOutput =
        new QPlainTextEdit(this);

    terminalOutput->setReadOnly(true);
    terminalOutput->setVisible(false);

    QFont terminalFont("Monospace");

    terminalFont.setStyleHint(
        QFont::TypeWriter
    );

    terminalOutput->setFont(
        terminalFont
    );

    layout->addWidget(
        terminalOutput
    );

    layout->addStretch();

    QHBoxLayout *creditLayout =
        new QHBoxLayout();

    creditLabel =
        new QLabel(
            "Created by <b>Seth Langer</b> &nbsp; | &nbsp; RELEASE: 1.1",
            this
        );

    creditLabel->setTextFormat(
        Qt::RichText
    );

    creditLabel->setStyleSheet(
        "font-size: 10px;"
        "font-style: italic;"
    );

    QPushButton *legalButton =
        new QPushButton(
            "i",
            this
        );

    legalButton->setFixedSize(18, 18);
    legalButton->setToolTip("License & Notices");
    legalButton->setAccessibleName("License & Notices");
    legalButton->setStyleSheet(
        "QPushButton {"
        "background-color: white;"
        "color: black;"
        "border: 1px solid #b0b0b0;"
        "border-radius: 9px;"
        "font-weight: bold;"
        "padding: 0px;"
        "}"
    );

    creditLayout->addWidget(
        legalButton
    );

    creditLayout->addWidget(
        creditLabel
    );

    creditLayout->addStretch();

    layout->addLayout(
        creditLayout
    );

    layout->addWidget(
        systemInfoButton
    );

    connect(
        legalButton,
        &QPushButton::clicked,
        this,
        [this]()
        {
            LegalNotices::show(this);
        }
    );

    loadTheme();
}

void MainWindow::applyTheme(const QString &themeName)
{
    QPalette palette;

    const auto applyTerminalPalette =
        [&palette](
            const QColor &window,
            const QColor &base,
            const QColor &control,
            const QColor &text,
            const QColor &disabled,
            const QColor &highlight
        )
        {
            palette.setColor(QPalette::Window, window);
            palette.setColor(QPalette::WindowText, text);
            palette.setColor(QPalette::Base, base);
            palette.setColor(QPalette::AlternateBase, control);
            palette.setColor(QPalette::ToolTipBase, text);
            palette.setColor(QPalette::ToolTipText, base);
            palette.setColor(QPalette::Text, text);
            palette.setColor(QPalette::Button, control);
            palette.setColor(QPalette::ButtonText, text);
            palette.setColor(QPalette::BrightText, Qt::red);
            palette.setColor(QPalette::Link, highlight);
            palette.setColor(QPalette::Highlight, highlight);
            palette.setColor(QPalette::HighlightedText, base);
            palette.setColor(QPalette::PlaceholderText, disabled);
            palette.setColor(QPalette::Disabled, QPalette::WindowText, disabled);
            palette.setColor(QPalette::Disabled, QPalette::Text, disabled);
            palette.setColor(QPalette::Disabled, QPalette::ButtonText, disabled);
        };

    if (themeName == "dark")
    {
        const QColor windowColor(45, 45, 45);
        const QColor controlColor(60, 60, 60);
        const QColor textColor(240, 240, 240);
        const QColor disabledTextColor(135, 135, 135);
        const QColor highlightColor(42, 130, 218);

        palette.setColor(QPalette::Window, windowColor);
        palette.setColor(QPalette::WindowText, textColor);
        palette.setColor(QPalette::Base, QColor(30, 30, 30));
        palette.setColor(QPalette::AlternateBase, windowColor);
        palette.setColor(QPalette::ToolTipBase, textColor);
        palette.setColor(QPalette::ToolTipText, QColor(20, 20, 20));
        palette.setColor(QPalette::Text, textColor);
        palette.setColor(QPalette::Button, controlColor);
        palette.setColor(QPalette::ButtonText, textColor);
        palette.setColor(QPalette::BrightText, Qt::red);
        palette.setColor(QPalette::Link, QColor(100, 170, 255));
        palette.setColor(QPalette::Highlight, highlightColor);
        palette.setColor(QPalette::HighlightedText, Qt::white);
        palette.setColor(QPalette::PlaceholderText, disabledTextColor);
        palette.setColor(
            QPalette::Disabled,
            QPalette::WindowText,
            disabledTextColor
        );
        palette.setColor(
            QPalette::Disabled,
            QPalette::Text,
            disabledTextColor
        );
        palette.setColor(
            QPalette::Disabled,
            QPalette::ButtonText,
            disabledTextColor
        );

        darkThemeAction->setChecked(true);
        creditLabel->setStyleSheet(
            "font-size: 10px;"
            "font-style: italic;"
            "color: #f0f0f0;"
        );
    }
    else if (themeName == "green-phosphor")
    {
        applyTerminalPalette(
            QColor(5, 18, 8),
            QColor(0, 10, 2),
            QColor(8, 30, 12),
            QColor(93, 255, 127),
            QColor(44, 128, 62),
            QColor(35, 170, 75)
        );
        greenThemeAction->setChecked(true);
        creditLabel->setStyleSheet(
            "font-size: 10px;"
            "font-style: italic;"
            "color: #5dff7f;"
        );
    }
    else if (themeName == "amber-crt")
    {
        applyTerminalPalette(
            QColor(24, 13, 0),
            QColor(12, 7, 0),
            QColor(42, 25, 4),
            QColor(255, 191, 64),
            QColor(145, 99, 25),
            QColor(205, 120, 20)
        );
        amberThemeAction->setChecked(true);
        creditLabel->setStyleSheet(
            "font-size: 10px;"
            "font-style: italic;"
            "color: #ffbf40;"
        );
    }
    else if (themeName == "classic-blue")
    {
        applyTerminalPalette(
            QColor(0, 20, 92),
            QColor(0, 8, 55),
            QColor(0, 34, 125),
            QColor(225, 232, 255),
            QColor(125, 145, 200),
            QColor(70, 130, 255)
        );
        blueThemeAction->setChecked(true);
        creditLabel->setStyleSheet(
            "font-size: 10px;"
            "font-style: italic;"
            "color: #e1e8ff;"
        );
    }
    else
    {
        palette.setColor(QPalette::Window, QColor(245, 245, 245));
        palette.setColor(QPalette::WindowText, QColor(20, 20, 20));
        palette.setColor(QPalette::Base, Qt::white);
        palette.setColor(QPalette::AlternateBase, QColor(235, 235, 235));
        palette.setColor(QPalette::ToolTipBase, QColor(255, 255, 220));
        palette.setColor(QPalette::ToolTipText, Qt::black);
        palette.setColor(QPalette::Text, QColor(20, 20, 20));
        palette.setColor(QPalette::Button, QColor(240, 240, 240));
        palette.setColor(QPalette::ButtonText, QColor(20, 20, 20));
        palette.setColor(QPalette::BrightText, Qt::red);
        palette.setColor(QPalette::Link, QColor(0, 90, 180));
        palette.setColor(QPalette::Highlight, QColor(48, 140, 230));
        palette.setColor(QPalette::HighlightedText, Qt::white);
        palette.setColor(QPalette::PlaceholderText, QColor(110, 110, 110));

        lightThemeAction->setChecked(true);
        creditLabel->setStyleSheet(
            "font-size: 10px;"
            "font-style: italic;"
            "color: #141414;"
        );
    }

    QApplication::setPalette(palette);

    QSettings settings(
        "Seth Langer",
        "Linux Update Utility"
    );
    settings.setValue("appearance/theme", themeName);
}

void MainWindow::loadTheme()
{
    QSettings settings(
        "Seth Langer",
        "Linux Update Utility"
    );

    const QString themeName =
        settings.value(
            "appearance/theme",
            "light"
        ).toString();

    const QStringList availableThemes =
    {
        "light",
        "dark",
        "green-phosphor",
        "amber-crt",
        "classic-blue"
    };

    applyTheme(
        availableThemes.contains(themeName)
            ? themeName
            : "light"
    );
}

void MainWindow::uninstallApplication()
{
    if (
        process &&
        process->state() != QProcess::NotRunning
    )
    {
        QMessageBox::warning(
            this,
            "Uninstall Unavailable",
            "Wait for the current system update to finish before "
            "uninstalling Linux Update Utility."
        );
        return;
    }

    if (!QFileInfo::exists(kInstalledAppImage))
    {
        QMessageBox::information(
            this,
            "Not Installed",
            "The system-wide Linux Update Utility installation was not "
            "found. This may be a development or portable copy."
        );
        return;
    }

    const QMessageBox::StandardButton choice =
        QMessageBox::warning(
            this,
            "Uninstall Linux Update Utility?",
            "This will remove Linux Update Utility, its application-menu "
            "entry, and its icon from this system.\n\n"
            "Your user-local application data will not be removed.",
            QMessageBox::Yes | QMessageBox::Cancel,
            QMessageBox::Cancel
        );

    if (choice != QMessageBox::Yes)
    {
        return;
    }

    bool accepted = false;
    QString password =
        PasswordDialog::getPassword(
            this,
            "Enter your password to uninstall Linux Update Utility:",
            &accepted
        );

    if (!accepted)
    {
        password.clear();
        return;
    }

    if (password.isEmpty())
    {
        QMessageBox::warning(
            this,
            "Authorization",
            "No password was entered."
        );
        return;
    }

    const QString uninstallCommand =
        "set -e\n"
        "rm -f -- \"$1\" \"$2\" \"$3\"\n";

    QProcess uninstaller;
    uninstaller.setProcessChannelMode(QProcess::MergedChannels);
    uninstaller.start(
        "sudo",
        QStringList()
            << "-S"
            << "-p"
            << ""
            << "bash"
            << "-c"
            << uninstallCommand
            << "luu-uninstaller"
            << kInstalledAppImage
            << kDesktopFile
            << kIconFile
    );

    if (!uninstaller.waitForStarted(3000))
    {
        password.clear();
        QMessageBox::critical(
            this,
            "Uninstall Failed",
            "Could not start administrator authorization."
        );
        return;
    }

    uninstaller.write(password.toUtf8() + QByteArray("\n"));
    uninstaller.closeWriteChannel();
    password.clear();

    if (!uninstaller.waitForFinished(-1) ||
        uninstaller.exitStatus() != QProcess::NormalExit ||
        uninstaller.exitCode() != 0)
    {
        QMessageBox::critical(
            this,
            "Uninstall Failed",
            "Linux Update Utility could not be removed. Check that your "
            "password is correct and try again."
        );
        return;
    }

    QMessageBox::information(
        this,
        "Uninstall Complete",
        "Linux Update Utility was removed successfully. The application "
        "will now close."
    );
    close();
}

bool MainWindow::generateScriptInternal()
{
    if (
        process &&
        process->state() !=
            QProcess::NotRunning
    )
    {
        return false;
    }

    QString workingDirectory =
        applicationRoot();

    QString outputPath =
        updateScriptPath();

    QFileInfo outputInfo(
        outputPath
    );

    if (!QDir().mkpath(
            outputInfo.absolutePath()
        ))
    {
        QMessageBox::critical(
            this,
            "Error",
            "Could not create the application's "
            "data directory."
        );

        return false;
    }

    QString backendPath =
        QCoreApplication::applicationDirPath()
        + "/linux-update-utility";

    if (!QFile::exists(
            backendPath
        ))
    {
        QMessageBox::critical(
            this,
            "Error",
            "The Linux Update Utility backend "
            "could not be found."
        );

        return false;
    }

    QProcess generator;

    generator.setWorkingDirectory(
        workingDirectory
    );

    QProcessEnvironment environment =
        QProcessEnvironment::systemEnvironment();

    environment.insert(
        "LINUX_UPDATE_SCRIPT_OUTPUT",
        outputPath
    );

    generator.setProcessEnvironment(
        environment
    );

    generator.start(
        backendPath
    );

    if (!generator.waitForStarted(
            3000))
    {
        QMessageBox::critical(
            this,
            "Error",
            "Failed to start the update utility."
        );

        return false;
    }

    if (!generator.waitForFinished(
            -1))
    {
        QMessageBox::critical(
            this,
            "Error",
            "The update utility failed to finish."
        );

        return false;
    }

    if (generator.exitCode() != 0)
    {
        QString error =
            QString::fromLocal8Bit(
                generator.readAllStandardError()
            );

        QMessageBox::critical(
            this,
            "Script Generation Failed",
            error.isEmpty()
                ? "The update utility returned an error."
                : error
        );

        return false;
    }

    if (!QFile::exists(
            outputPath
        ))
    {
        QMessageBox::critical(
            this,
            "Script Generation Failed",
            "The update utility completed, but the "
            "generated script could not be found."
        );

        return false;
    }

    return true;
}

void MainWindow::generateScript()
{
    if (generateScriptInternal())
    {
        QMessageBox::information(
            this,
            "Script Generated",
            "The update script was generated successfully."
        );
    }
}

bool MainWindow::scriptRequiresSudo() const
{
    QFile script(
        updateScriptPath()
    );

    if (!script.open(
            QIODevice::ReadOnly |
            QIODevice::Text
        ))
    {
        return false;
    }

    QString contents =
        QString::fromUtf8(
            script.readAll()
        );

    return contents.contains(
        "sudo -S"
    );
}

bool MainWindow::requestSudoPassword()
{
    bool accepted = false;

    QString password =
        PasswordDialog::getPassword(
            this,
            "Enter your password to authorize system updates:",
            &accepted
        );

    if (!accepted)
    {
        return false;
    }

    if (password.isEmpty())
    {
        QMessageBox::warning(
            this,
            "Authorization",
            "No password was entered."
        );

        return false;
    }

    sudoPassword =
        password;

    password.clear();

    return true;
}

void MainWindow::updateSystem()
{
    if (
        process &&
        process->state() !=
            QProcess::NotRunning
    )
    {
        return;
    }

    if (!generateScriptInternal())
    {
        return;
    }

    QString scriptPath =
        updateScriptPath();

    if (!QFile::exists(
            scriptPath
        ))
    {
        QMessageBox::critical(
            this,
            "Update Failed",
            "The update script could not be found."
        );

        return;
    }

    sudoPassword.clear();

    if (scriptRequiresSudo())
    {
        if (!requestSudoPassword())
        {
            return;
        }
    }

    terminalOutput->clear();
    terminalOutput->setVisible(true);

    generateButton->setEnabled(
        false
    );

    updateButton->setEnabled(
        false
    );

    systemInfoButton->setEnabled(
        false
    );

    uninstallAction->setEnabled(
        false
    );

    resize(
        850,
        600
    );

    terminalOutput->appendPlainText(
        "Linux Update Utility"
    );

    terminalOutput->appendPlainText(
        "===================="
    );

    terminalOutput->appendPlainText(
        "Starting system update..."
    );

    terminalOutput->appendPlainText("");

    startUpdateProcess();
}

void MainWindow::startUpdateProcess()
{
    if (process)
    {
        return;
    }

    QString workingDirectory =
        applicationRoot();

    QString scriptPath =
        updateScriptPath();

    process =
        new QProcess();

    process->setWorkingDirectory(
        workingDirectory
    );

    process->setProcessChannelMode(
        QProcess::MergedChannels
    );

    connect(
        process,
        &QProcess::readyReadStandardOutput,
        this,
        &MainWindow::appendTerminalOutput
    );

    connect(
        process,
        &QProcess::errorOccurred,
        this,
        [this](QProcess::ProcessError)
        {
            if (!process)
            {
                return;
            }

            terminalOutput->appendPlainText(
                "\nERROR: Update process encountered an error."
            );
        }
    );

    connect(
        process,
        &QProcess::finished,
        this,
        [this](
            int exitCode,
            QProcess::ExitStatus exitStatus
        )
        {
            if (!process)
            {
                return;
            }

            appendTerminalOutput();

            terminalOutput->appendPlainText("");

            terminalOutput->appendPlainText(
                "========================================"
            );

            if (
                exitStatus ==
                    QProcess::NormalExit &&
                exitCode == 0
            )
            {
                terminalOutput->appendPlainText(
                    "Update completed successfully."
                );
            }
            else if (
                exitStatus ==
                    QProcess::NormalExit
            )
            {
                terminalOutput->appendPlainText(
                    QString(
                        "Update finished with exit code %1."
                    ).arg(exitCode)
                );
            }
            else
            {
                terminalOutput->appendPlainText(
                    "Update process terminated unexpectedly."
                );
            }

            terminalOutput->appendPlainText(
                "========================================"
            );

            sudoPassword.clear();

            generateButton->setEnabled(
                true
            );

            updateButton->setEnabled(
                true
            );

            systemInfoButton->setEnabled(
                true
            );

            uninstallAction->setEnabled(
                true
            );

            QProcess *finishedProcess =
                process;

            process = nullptr;

            finishedProcess->deleteLater();
        }
    );

    process->start(
        "bash",
        QStringList()
            << scriptPath
    );

    if (!process->waitForStarted(
            3000))
    {
        terminalOutput->appendPlainText(
            "ERROR: Could not start the update script."
        );

        sudoPassword.clear();

        QProcess *failedProcess =
            process;

        process = nullptr;

        generateButton->setEnabled(
            true
        );

        updateButton->setEnabled(
            true
        );

        systemInfoButton->setEnabled(
            true
        );

        uninstallAction->setEnabled(
            true
        );

        failedProcess->deleteLater();

        return;
    }

    /*
     * The generated script uses exactly one sudo -S
     * invocation for privileged package-manager operations.
     */
    if (!sudoPassword.isEmpty())
    {
        process->write(
            sudoPassword.toUtf8()
            + QByteArray("\n")
        );

        process->closeWriteChannel();

        sudoPassword.clear();
    }
    else
    {
        process->closeWriteChannel();
    }
}

void MainWindow::appendTerminalOutput()
{
    if (!process)
    {
        return;
    }

    QByteArray output =
        process->readAllStandardOutput();

    if (output.isEmpty())
    {
        return;
    }

    terminalOutput->insertPlainText(
        QString::fromLocal8Bit(
            output
        )
    );

    terminalOutput->moveCursor(
        QTextCursor::End
    );
}

void MainWindow::showSystemInfo()
{
    QString workingDirectory =
        applicationRoot();

    QString distribution =
        "Unknown";

    QFile osRelease(
        "/etc/os-release"
    );

    if (
        osRelease.open(
            QIODevice::ReadOnly |
            QIODevice::Text
        )
    )
    {
        while (!osRelease.atEnd())
        {
            QString line =
                QString::fromUtf8(
                    osRelease.readLine()
                ).trimmed();

            if (
                line.startsWith(
                    "PRETTY_NAME="
                )
            )
            {
                distribution =
                    line.mid(
                        QString(
                            "PRETTY_NAME="
                        ).length()
                    ).remove('"');

                break;
            }
        }
    }

    QString kernel =
        QSysInfo::kernelVersion();

    QString architecture =
        QSysInfo::currentCpuArchitecture();

    QStringList packageManagers;

    QProcess detector;

    detector.setWorkingDirectory(
        workingDirectory
    );

    detector.start(
        "bash",
        QStringList()
            << "./scripts/detect-package-managers.sh"
    );

    if (
        detector.waitForStarted(3000) &&
        detector.waitForFinished(3000)
    )
    {
        QString output =
            QString::fromUtf8(
                detector.readAllStandardOutput()
            );

        const QStringList lines =
            output.split(
                '\n',
                Qt::SkipEmptyParts
            );

        for (
            const QString &line :
            lines
        )
        {
            QString manager =
                QFileInfo(
                    line.trimmed()
                ).fileName();

            if (!manager.isEmpty())
            {
                packageManagers.append(
                    manager
                );
            }
        }
    }

    QString packageManagerText =
        packageManagers.isEmpty()
            ? "None detected"
            : packageManagers.join(
                  ", "
              );

    QString scriptStatus =
        QFile::exists(
            updateScriptPath()
        )
            ? "Generated"
            : "Not generated";

    QDialog dialog(this);

    dialog.setWindowTitle(
        "System Info"
    );

    dialog.resize(
        450,
        280
    );

    QFormLayout *layout =
        new QFormLayout(
            &dialog
        );

    layout->addRow(
        "Distribution:",
        new QLabel(
            distribution
        )
    );

    layout->addRow(
        "Architecture:",
        new QLabel(
            architecture
        )
    );

    layout->addRow(
        "Kernel:",
        new QLabel(
            kernel
        )
    );

    layout->addRow(
        "Package Managers:",
        new QLabel(
            packageManagerText
        )
    );

    layout->addRow(
        "Update Script:",
        new QLabel(
            scriptStatus
        )
    );

    dialog.exec();
}
