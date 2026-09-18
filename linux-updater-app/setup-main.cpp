#include <QApplication>
#include <QIcon>

#include "SetupWindow.hpp"

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);

    app.setApplicationName("luu-setup-utility");
    app.setApplicationDisplayName("LUU Setup Utility");
    app.setWindowIcon(QIcon(":/luu-icon.svg"));

    SetupWindow window;
    window.show();

    return app.exec();
}
