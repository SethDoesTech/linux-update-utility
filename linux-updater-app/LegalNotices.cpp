#include "LegalNotices.hpp"

#include <QDialog>
#include <QDialogButtonBox>
#include <QLabel>
#include <QVBoxLayout>

void LegalNotices::show(QWidget *parent)
{
    QDialog dialog(parent);
    dialog.setWindowTitle("License Notice");
    dialog.setFixedSize(330, 150);

    QVBoxLayout *layout = new QVBoxLayout(&dialog);
    QLabel *notice = new QLabel(
        "Copyright © 2026 Seth Langer<br>"
        "Licensed under GNU GPLv3<br>"
        "Linux Update Utility",
        &dialog
    );
    notice->setTextFormat(Qt::RichText);
    notice->setAlignment(Qt::AlignCenter);
    layout->addWidget(notice);

    QDialogButtonBox *buttons = new QDialogButtonBox(
        QDialogButtonBox::Close,
        &dialog
    );

    QObject::connect(
        buttons,
        &QDialogButtonBox::rejected,
        &dialog,
        &QDialog::close
    );

    layout->addWidget(buttons);
    dialog.exec();
}
