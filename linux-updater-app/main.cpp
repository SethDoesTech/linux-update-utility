#include <QApplication>
#include <QIcon>

#include "MainWindow.hpp"

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);

    app.setApplicationName(
        "linux-update-utility"
    );

    app.setApplicationDisplayName(
        "Linux Update Utility"
    );

    app.setDesktopFileName(
        "linux-update-utility"
    );

    app.setWindowIcon(
        QIcon(":/luu-icon.svg")
    );

    MainWindow window;
    window.show();

    return app.exec();
}