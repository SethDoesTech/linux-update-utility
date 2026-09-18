#ifndef SETUPWINDOW_HPP
#define SETUPWINDOW_HPP

#include <QMainWindow>

class QLabel;
class QPushButton;

class SetupWindow : public QMainWindow
{
public:
    SetupWindow();

private:
    QString appImageSourcePath() const;
    bool isInstalled() const;
    bool installApplication();
    bool requestAdministratorPassword(QString &password);
    void showAlreadyInstalled();

    QLabel *messageLabel;
    QPushButton *yesButton;
    QPushButton *noButton;
};

#endif
