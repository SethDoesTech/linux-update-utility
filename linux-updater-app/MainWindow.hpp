#ifndef MAINWINDOW_HPP
#define MAINWINDOW_HPP

#include <QMainWindow>
#include <QString>

class QPushButton;
class QProcess;
class QPlainTextEdit;

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

    bool scriptRequiresSudo() const;
    bool requestSudoPassword();

    QString applicationRoot() const;
    QString updateScriptPath() const;

    QPushButton *generateButton;
    QPushButton *updateButton;
    QPushButton *systemInfoButton;

    QPlainTextEdit *terminalOutput;

    QProcess *process;

    QString sudoPassword;
};

#endif