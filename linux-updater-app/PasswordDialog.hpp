#pragma once

#include <QEvent>
#include <QGuiApplication>
#include <QInputDialog>
#include <QInputMethod>
#include <QLineEdit>
#include <QTimer>

// Request the desktop keyboard explicitly: mouse focus does not necessarily
// open it automatically on desktops such as KDE Plasma.
class PasswordDialog : public QInputDialog
{
public:
    PasswordDialog(QWidget *parent, const QString &prompt)
        : QInputDialog(parent)
    {
        setWindowTitle("Administrator Authorization");
        setInputMode(QInputDialog::TextInput);
        setLabelText(prompt);
        setTextEchoMode(QLineEdit::Password);
        setInputMethodHints(Qt::ImhHiddenText | Qt::ImhSensitiveData
                            | Qt::ImhNoPredictiveText | Qt::ImhNoAutoUppercase);
        editor = findChild<QLineEdit *>();
        if (editor)
            editor->installEventFilter(this);
    }

    static QString getPassword(QWidget *parent, const QString &prompt,
                               bool *accepted)
    {
        PasswordDialog dialog(parent, prompt);
        *accepted = dialog.exec() == QDialog::Accepted;
        return *accepted ? dialog.textValue() : QString();
    }

protected:
    bool eventFilter(QObject *watched, QEvent *event) override
    {
        if (watched == editor
            && (event->type() == QEvent::FocusIn
                || event->type() == QEvent::MouseButtonRelease))
        {
            // Let Qt finish assigning focus before asking the input method.
            // Binding the callback to this dialog also cancels it on deletion.
            QTimer::singleShot(0, this, [this] {
                if (isVisible() && editor->hasFocus())
                    QGuiApplication::inputMethod()->show();
            });
        }
        return QInputDialog::eventFilter(watched, event);
    }

private:
    QLineEdit *editor = nullptr;
};
