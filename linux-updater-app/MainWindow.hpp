#ifndef MAINWINDOW_HPP
#define MAINWINDOW_HPP

#include <QMainWindow>
#include <QString>

class QPushButton;
class QProcess;
class QPlainTextEdit;
class QAction;
class QLabel;

class MainWindow : public QMainWindow
{
public:
    MainWindow();

private:
    void setupUI();

    bool generateScriptInternal();
    void generateScript();

    void updateSystem();
    void startUpdateProcess();
    void appendTerminalOutput();

    void showSystemInfo();
    void uninstallApplication();
    void applyTheme(const QString &themeName);
    void loadTheme();

    bool scriptRequiresSudo() const;
    bool requestSudoPassword();

    QString applicationRoot() const;
    QString updateScriptPath() const;

    QPushButton *generateButton;
    QPushButton *updateButton;
    QPushButton *systemInfoButton;
    QAction *uninstallAction;
    QAction *lightThemeAction;
    QAction *darkThemeAction;
    QAction *greenThemeAction;
    QAction *amberThemeAction;
    QAction *blueThemeAction;
    QLabel *creditLabel;

    QPlainTextEdit *terminalOutput;

    QProcess *process;

    QString sudoPassword;
};

#endif
