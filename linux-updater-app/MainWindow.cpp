#include "MainWindow.hpp"
#include "LegalNotices.hpp"

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
#include <QInputDialog>
#include <QLineEdit>
#include <QFont>
#include <QTextCursor>
#include <QStandardPaths>
#include <QProcessEnvironment>

MainWindow::MainWindow()
    : process(nullptr)
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

    QLabel *creditLabel =
        new QLabel(
            "Created by <b>Seth Langer</b>",
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
        QInputDialog::getText(
            this,
            "Administrator Authorization",
            "Enter your password to authorize system updates:",
            QLineEdit::Password,
            QString(),
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
