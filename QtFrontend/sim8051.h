#ifndef SIM8051_H
#define SIM8051_H

#include <QMainWindow>
#include <QLineEdit>
#include <QPlainTextEdit>
#include <QWidget>
#include <iostream>
#include <memory>
#include <QPushButton>
#include <QBoxLayout>
#include <QFile>
#include <QShortcut>

#include "asm8051.h"

#define ThisType std::remove_reference_t<decltype(*this)>

QT_BEGIN_NAMESPACE
namespace Ui { class Sim8051; }
QT_END_NAMESPACE

class CodeEditor : public QPlainTextEdit
{
    Q_OBJECT
public:
    CodeEditor(QWidget* parent = nullptr);

    int lineNumberAreaWidth();
    void lineNumberAreaPaintEvent(QPaintEvent *event);

    QWidget* lineNumberArea;
    void resizeEvent(QResizeEvent* event) override;

protected:

private:
   private slots:
    void updateLineNumberAreaWidth(int newBlockSize);
    void hightlightCurrentLine();
    void updateLineNumberArea(const QRect& rect, int dy);
};

class LineNumberArea : public QWidget
{
public:
    LineNumberArea(CodeEditor * editor) : QWidget(editor), codeEditor(editor)
    {

    }
    QSize sizeHint() const override
    {
        return QSize(0,0);
    }
public:
    void paintEvent(QPaintEvent* event) override {
        codeEditor->lineNumberAreaPaintEvent(event);
    }
private:
    CodeEditor* codeEditor;
};


// New widget to handle other UI related things

class Assembler : public QWidget
{
    Q_OBJECT
public :
    Assembler(QWidget* parent) : QWidget(parent)
    {

        asm_widget = new AsmOutData();

        editor = new CodeEditor(this);

        compile_button = new QPushButton(this);
        compile_button->setText("Simulate/Compile");
        compile_button->setToolTip("What else would it do?");

        run_button = new QPushButton(this);
        run_button->setText("Run");
        run_button->setToolTip("Don't know its work...");

        load_button = new QPushButton(this);
        load_button->setText("Load file");
//        auto *grid = new QGridLayout(this);
//        grid->addWidget(compile_button,0,0);
//        grid->addWidget(run_button,0,1);
//        grid->addWidget(editor,1,0,1,2);

        compile_button->setMaximumSize(1500,60);
        run_button->setMaximumSize(1500,60);
        load_button->setMaximumHeight(50);

        compile_button->setAutoFillBackground(true);
        auto palette = compile_button->palette();
        palette.setColor(QPalette::Window,QColor(Qt::blue));

        run_button->setAutoFillBackground(true);
        compile_button->setPalette(palette);
        compile_button->update();

        palette.setColor(QPalette::Window,QColor(Qt::magenta));
        run_button->setPalette(palette);
//        compile_button->setStyleSheet("background:qlineargradient(x1:0, y1:0, x2:0, y2:1, "
//                                      "stop : 0 black, stop : 0.4 green, stop : 0.5 darkgray, stop : 1.0 black);"
//                                      "color:red; font-size:24px");

        auto hLayout = new QHBoxLayout();
        hLayout->addWidget(load_button);
        hLayout->addWidget(compile_button,1);
        hLayout->addWidget(run_button,1);


        auto vLayout = new QVBoxLayout(this);
        vLayout->addLayout(hLayout,1);
        vLayout->addWidget(editor);

        connect(load_button,&QPushButton::pressed,this,&ThisType::FileLoader);
        connect(compile_button,&QPushButton::pressed,this,&ThisType::Compile);
        auto save_shortcut = new QShortcut(QKeySequence("Ctrl+S"),this);
        connect(save_shortcut,&QShortcut::activated,this,&ThisType::FileSaver);
    }

        ~Assembler() {
            delete asm_widget;
        }

public slots:
    void FileLoader();
    void FileSaver();
    void Compile();

private:
    std::unique_ptr<QFile> currentFilePtr = nullptr;
    CodeEditor * editor;
    QPushButton* compile_button;
    QPushButton* run_button;
    QPushButton* load_button;
    AsmOutData *  asm_widget;
};

class Sim8051 : public QMainWindow
{
    Q_OBJECT

public:
    Sim8051(QWidget *parent = nullptr);
    ~Sim8051();

private:
    Ui::Sim8051 *ui;
    Assembler* asm8051;
};

#endif // SIM8051_H
