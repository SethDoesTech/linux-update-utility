#include "SetupWindow.hpp"
#include "LegalNotices.hpp"

#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QHBoxLayout>
#include <QInputDialog>
#include <QLabel>
#include <QLineEdit>
#include <QMessageBox>
#include <QProcess>
#include <QPushButton>
#include <QTemporaryFile>
#include <QTextStream>
#include <QVBoxLayout>
#include <QWidget>

namespace
{
const QString kInstalledAppImage =
    "/usr/local/bin/linux-update-utility.AppImage";

const QString kDesktopFile =
    "/usr/share/applications/linux-update-utility.desktop";

const QString kIconFile =
    "/usr/share/icons/hicolor/scalable/apps/linux-update-utility.svg";
}

SetupWindow::SetupWindow()
{
    setWindowTitle("LUU Setup Utility");
    setFixedSize(560, 330);

    QWidget *centralWidget = new QWidget(this);
    setCentralWidget(centralWidget);

    QVBoxLayout *layout = new QVBoxLayout(centralWidget);
    layout->setContentsMargins(38, 32, 38, 24);
    layout->setSpacing(18);

    messageLabel = new QLabel(this);
    messageLabel->setWordWrap(true);
    messageLabel->setText(
        "Would you like to set up Linux Update Utility?\n\n"
        "If yes, the app will be available like any other application. "
        "To get started, press \"Generate Script\" and then "
        "\"Update System\".\n\n"
        "If no, the setup utility will make no changes to your system."
    );
    layout->addWidget(messageLabel);
    layout->addStretch();

    QHBoxLayout *buttonLayout = new QHBoxLayout();
    buttonLayout->addStretch();

    yesButton = new QPushButton("Yes, Set Up", this);
    noButton = new QPushButton("No, Do Nothing", this);
    yesButton->setFixedWidth(150);
    noButton->setFixedWidth(150);

    buttonLayout->addWidget(yesButton);
    buttonLayout->addWidget(noButton);
    layout->addLayout(buttonLayout);

    QHBoxLayout *creditLayout = new QHBoxLayout();
    QLabel *creditLabel = new QLabel(
        "Created by <b>Seth Langer</b> &nbsp; | &nbsp; RELEASE: 1.0",
        this
    );
    creditLabel->setTextFormat(Qt::RichText);
    creditLabel->setStyleSheet(
        "font-size: 10px;"
        "font-style: italic;"
    );
    QPushButton *legalButton = new QPushButton("i", this);
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
    creditLayout->addWidget(legalButton);
    creditLayout->addWidget(creditLabel);

    creditLayout->addStretch();
    layout->addLayout(creditLayout);

    connect(
        yesButton,
        &QPushButton::clicked,
        this,
        [this]()
        {
            installApplication();
        }
    );

    connect(
        noButton,
        &QPushButton::clicked,
        this,
        &QWidget::close
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

    if (isInstalled())
    {
        showAlreadyInstalled();
    }
}

QString SetupWindow::appImageSourcePath() const
{
    const QString applicationDirectory =
        QCoreApplication::applicationDirPath();

    const QString embeddedPayload =
        QDir(applicationDirectory).absoluteFilePath(
            "../share/linux-update-utility/payload/"
            "Linux_Update_Utility-x86_64.AppImage"
        );

    if (QFileInfo::exists(embeddedPayload))
    {
        return embeddedPayload;
    }

    /*
     * Keep local development and two-file testing simple. The release setup
     * AppImage uses the embedded payload path above.
     */
    return QDir(applicationDirectory).filePath(
        "Linux_Update_Utility-x86_64.AppImage"
    );
}

bool SetupWindow::isInstalled() const
{
    return QFileInfo::exists(kInstalledAppImage);
}

bool SetupWindow::requestAdministratorPassword(QString &password)
{
    bool accepted = false;

    password = QInputDialog::getText(
        this,
        "Administrator Authorization",
        "Enter your password to install Linux Update Utility:",
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

    return true;
}

bool SetupWindow::installApplication()
{
    const bool updatingExistingInstall = isInstalled();
    const QString sourcePath = appImageSourcePath();

    if (!QFileInfo::exists(sourcePath))
    {
        QMessageBox::critical(
            this,
            "Setup Failed",
            "The Linux Update Utility AppImage payload could not "
            "be found."
        );
        return false;
    }

    /*
     * An embedded payload lives inside the setup AppImage's read-only FUSE
     * mount. Stage it in a normal user-writable temporary file before sudo
     * reads it, so installation does not depend on root accessing that mount.
     */
    QFile sourceAppImage(sourcePath);
    if (!sourceAppImage.open(QIODevice::ReadOnly))
    {
        QMessageBox::critical(
            this,
            "Setup Failed",
            "Could not read the Linux Update Utility AppImage payload."
        );
        return false;
    }

    QTemporaryFile stagedAppImage(
        QDir::tempPath() + "/linux-update-utility-payload-XXXXXX.AppImage"
    );

    if (!stagedAppImage.open())
    {
        QMessageBox::critical(
            this,
            "Setup Failed",
            "Could not prepare the Linux Update Utility AppImage."
        );
        return false;
    }

    while (!sourceAppImage.atEnd())
    {
        const QByteArray data = sourceAppImage.read(1024 * 1024);

        if (data.isEmpty() && sourceAppImage.error() != QFile::NoError)
        {
            QMessageBox::critical(
                this,
                "Setup Failed",
                "Could not read the Linux Update Utility AppImage payload."
            );
            return false;
        }

        if (stagedAppImage.write(data) != data.size())
        {
            QMessageBox::critical(
                this,
                "Setup Failed",
                "Could not stage the Linux Update Utility AppImage."
            );
            return false;
        }
    }

    if (!stagedAppImage.flush())
    {
        QMessageBox::critical(
            this,
            "Setup Failed",
            "Could not finish preparing the Linux Update Utility AppImage."
        );
        return false;
    }

    stagedAppImage.close();

    QTemporaryFile iconFile(
        QDir::tempPath() + "/linux-update-utility-icon-XXXXXX.svg"
    );

    if (!iconFile.open())
    {
        QMessageBox::critical(
            this,
            "Setup Failed",
            "Could not prepare the application icon."
        );
        return false;
    }

    QFile bundledIcon(":/luu-icon.svg");
    if (!bundledIcon.open(QIODevice::ReadOnly) ||
        iconFile.write(bundledIcon.readAll()) < 0)
    {
        QMessageBox::critical(
            this,
            "Setup Failed",
            "Could not read the bundled application icon."
        );
        return false;
    }

    iconFile.flush();

    QTemporaryFile desktopFile(
        QDir::tempPath() + "/linux-update-utility-XXXXXX.desktop"
    );

    if (!desktopFile.open())
    {
        QMessageBox::critical(
            this,
            "Setup Failed",
            "Could not prepare the desktop launcher."
        );
        return false;
    }

    QTextStream desktopEntry(&desktopFile);
    desktopEntry
        << "[Desktop Entry]\n"
        << "Type=Application\n"
        << "Name=Linux Update Utility\n"
        << "Comment=Update your Linux system through a graphical interface\n"
        << "Exec=" << kInstalledAppImage << "\n"
        << "Icon=" << kIconFile << "\n"
        << "Terminal=false\n"
        << "Categories=System;Settings;\n"
        << "StartupNotify=true\n"
        << "StartupWMClass=linux-update-utility\n";

    desktopEntry.flush();
    desktopFile.flush();

    QString password;
    if (!requestAdministratorPassword(password))
    {
        password.clear();
        return false;
    }

    const QString installCommand =
        "set -e\n"
        "install -Dm755 \"$1\" \"${2}.new\"\n"
        "mv -f \"${2}.new\" \"$2\"\n"
        "install -Dm644 \"$3\" \"$4\"\n"
        "install -Dm644 \"$5\" \"$6\"\n";

    QProcess installer;
    installer.setProcessChannelMode(QProcess::MergedChannels);
    installer.start(
        "sudo",
        QStringList()
            << "-S"
            << "-p"
            << ""
            << "bash"
            << "-c"
            << installCommand
            << "luu-installer"
            << stagedAppImage.fileName()
            << kInstalledAppImage
            << iconFile.fileName()
            << kIconFile
            << desktopFile.fileName()
            << kDesktopFile
    );

    if (!installer.waitForStarted(3000))
    {
        password.clear();
        QMessageBox::critical(
            this,
            "Setup Failed",
            "Could not start administrator authorization."
        );
        return false;
    }

    installer.write(password.toUtf8() + QByteArray("\n"));
    installer.closeWriteChannel();
    password.clear();

    if (!installer.waitForFinished(-1) || installer.exitCode() != 0)
    {
        QMessageBox::critical(
            this,
            updatingExistingInstall ? "Update Failed" : "Setup Failed",
            updatingExistingInstall
                ? "Linux Update Utility could not be updated. "
                  "Check that your password is correct and try again."
                : "Linux Update Utility could not be installed. "
                  "Check that your password is correct and try again."
        );
        return false;
    }

    if (!QProcess::startDetached(kInstalledAppImage))
    {
        QMessageBox::warning(
            this,
            updatingExistingInstall ? "Update Complete" : "Setup Complete",
            updatingExistingInstall
                ? "Linux Update Utility was updated, but could not be "
                  "started automatically. You can try opening it from your "
                  "applications menu."
                : "Linux Update Utility was installed, but could not be "
                "started automatically. You can try opening it from your "
                  "applications menu."
        );
        close();
        return true;
    }

    QMessageBox::information(
        this,
        updatingExistingInstall ? "Update Complete" : "Setup Complete",
        updatingExistingInstall
            ? "Linux Update Utility was updated and is available from your "
              "applications menu."
            : "Linux Update Utility is installed and available from your "
              "applications menu."
    );

    close();
    return true;
}

void SetupWindow::showAlreadyInstalled()
{
    messageLabel->setText(
        "Linux Update Utility is already installed on this system.\n\n"
        "Select \"Update Existing Install\" to replace it with the "
        "version included in this setup utility."
    );
    yesButton->setText("Update Existing Install");
    yesButton->setVisible(true);
    noButton->setText("Close");
}
