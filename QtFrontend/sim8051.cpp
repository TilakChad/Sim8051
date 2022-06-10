#include "./sim8051.h"
#include "./ui_sim8051.h"
#include <iostream>
#include <QPainter>
#include <QTextBlock>
#include <QFile>
#include <QTextStream>
#include <QBoxLayout>
#include <QFileDialog>
#include "./asm8051.h"

Sim8051::Sim8051(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::Sim8051)
{
    ui->setupUi(this);
    asm8051 = new Assembler(this);
    setCentralWidget(asm8051);

}

Sim8051::~Sim8051()
{
    delete ui;
}

CodeEditor::CodeEditor(QWidget* parent) : QPlainTextEdit(parent)
{
    lineNumberArea = new LineNumberArea(this);
    // update request is periodic it seems
    connect(this,&CodeEditor::updateRequest,this,&CodeEditor::updateLineNumberArea);
    connect(this,&CodeEditor::blockCountChanged,this,&CodeEditor::updateLineNumberAreaWidth);
    connect(this,&CodeEditor::cursorPositionChanged,this,&CodeEditor::hightlightCurrentLine);
    updateLineNumberAreaWidth(0);
    QFont font;
    font.setFamily("Fira Code Retina");
    font.setPointSize(22);
//    document()->setDefaultFont(font);
    setFont(font);
    // Load file
    QFile file(":test.asm");
    file.open(QFile::ReadOnly);
    QTextStream in(&file);
    in.autoDetectUnicode();
    auto contents = in.readAll();
    this->document()->setPlainText(contents);
}

int CodeEditor::lineNumberAreaWidth()
{
    int digits = 1;
    int max = std::max(1,blockCount());

    while (max >= 10) {
      max /= 10;
      ++digits;
    }
    return 10 + fontMetrics().horizontalAdvance(QLatin1Char('9')) * digits;
}

void CodeEditor::updateLineNumberAreaWidth(int) {
  setViewportMargins(lineNumberAreaWidth() + 10, 0, 0, 0);
}

void CodeEditor::updateLineNumberArea(const QRect &rect, int dy) {
  if (dy) {
    lineNumberArea->scroll(0, dy);
  } else {
    lineNumberArea->update(0, rect.y(), lineNumberArea->width(), rect.height());
  }
}

void CodeEditor::hightlightCurrentLine() {
  QList<QTextEdit::ExtraSelection> extraSelection;
  if (!this->isReadOnly()) {
    QTextEdit::ExtraSelection selection;
    QColor lineColor = QColor(Qt::green).lighter(150);

    selection.format.setBackground(lineColor);
    selection.format.setProperty(QTextFormat::FullWidthSelection, true);
    selection.cursor = textCursor();
    selection.cursor.clearSelection();
    extraSelection.append(selection);
  }
  setExtraSelections(extraSelection);
}

void CodeEditor::lineNumberAreaPaintEvent(QPaintEvent *event) {
  QPainter painter{lineNumberArea};
  std::cerr << "lineNumberRepainEvent generated" << std::endl;
  painter.fillRect(event->rect(), Qt::lightGray);

  auto block = firstVisibleBlock();
  int blockNumber = block.blockNumber();
  int top = blockBoundingGeometry(block).translated(contentOffset()).top();
  int bottom = blockBoundingRect(block).height() + top;

  while (block.isValid() && top <= event->rect().bottom()) {
    if (block.isVisible() && bottom >= event->rect().top()) {
      QString number = QString::number(blockNumber + 1);
      painter.setPen(Qt::black);

      painter.drawText(0, top, lineNumberAreaWidth(), fontMetrics().height(),
                       Qt::AlignRight, number);
    }

    block = block.next();
    top = bottom;
    bottom = top + blockBoundingRect(block).height();
    ++blockNumber;
  }
}

void CodeEditor::resizeEvent(QResizeEvent *event) {
  QPlainTextEdit::resizeEvent(event);
  QRect cr = this->contentsRect();

  lineNumberArea->setGeometry(
      QRect(cr.left(), cr.top(), lineNumberAreaWidth(), cr.height()));
}

// Assembler

void Assembler::FileLoader() {
  auto fileName = QFileDialog::getOpenFileName(
      this, "asm source file", "", "asm source (*.asm);;All Files(*)");
  if (fileName.isEmpty())
    return;
  currentFilePtr = std::unique_ptr<QFile>(new QFile(fileName));
  if (!currentFilePtr->open(QFile::ReadWrite)) {
    std::cerr << "Failed to load " << fileName.toStdString() << " due to "
              << std::endl;
    std::cerr << currentFilePtr->errorString().toStdString();
    currentFilePtr = nullptr;
    return;
  }
  // Else read everything from the file
  QTextStream stream(currentFilePtr.get());
  stream.autoDetectUnicode();
  editor->document()->setPlainText(stream.readAll());
}

void Assembler::FileSaver() {
  if (!currentFilePtr)
    return;
  auto content = editor->document()->toPlainText();
  currentFilePtr->seek(0);
  QTextStream stream(currentFilePtr.get());
  stream.autoDetectUnicode();
  stream << content;
}

void Assembler::Compile() {
  auto content = editor->document()->toPlainText();
  auto stdcontent = content.toStdString();
  // std::cerr << "Reached here" << std::endl;
  auto ffi_data = RustAssemble(stdcontent.c_str(),
                               static_cast<uint64_t>(stdcontent.length()));
  if (ffi_data.compiled) {
    std::cerr << "Program compiled successfully";
  }

  std::vector<uint8_t> regs(ffi_data.reg_arr,
                            ffi_data.reg_arr + ffi_data.reg_len);
  asm_widget->UpdateReg(regs);

  for (int i= 0; i < ffi_data.sfr_len; ++i)
      std::cerr << "SFR " << i << " : " << (int) ffi_data.sfr_arr[i] << std::endl;
  std::vector<uint8_t> sfrs(ffi_data.sfr_arr,
                            ffi_data.sfr_arr + ffi_data.sfr_len);


  std::vector<uint8_t> memory(ffi_data.memory, ffi_data.memory + ffi_data.memory_len);
  asm_widget->UpdateSFR(sfrs);
  asm_widget->UpdateMem(memory);
  // Create a new widget and paint everything from here to there
  asm_widget->showNormal();
}
