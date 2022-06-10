#include "sim8051.h"
#include "asm8051.h"

#include <QApplication>
#include <QPushButton>

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);
    Sim8051 w;
    w.show();
    // AsmOutData ws;
    // ws.showMaximized();
    return a.exec();
}
