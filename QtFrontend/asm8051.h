#ifndef ASM8051_H
#define ASM8051_H

#include <qnamespace.h>
extern "C" {
#include <stdint.h>
#include <stdbool.h>
    struct AsmData {
        bool compiled;
        uint8_t psw;
        uint8_t* sfr_arr;
        uint64_t sfr_len;
        uint8_t* reg_arr;
        uint64_t reg_len;
        uint8_t* memory;
        uint64_t memory_len;
    };
    AsmData RustAssemble(const char* , uint64_t len);
}

#include <QWidget>
#include <QLabel>
#include <vector>
#include <QDockWidget>
#include <QBoxLayout>

#include <iomanip>
#include <ranges>

class AsmOutData : public QWidget {

    public:
        AsmOutData(QWidget* parent = nullptr) : QWidget(parent) {
            sfr_label = new QLabel(this);
            reg_label = new QLabel(this);

            QFont font;
            font.setFamily("Fira Code Retina");
            font.setPointSize(22);

            setFont(font);

            auto hlayout = new QHBoxLayout();
            flag_label = new QLabel(this);
            hlayout->addWidget(flag_label);
            hlayout->addWidget(reg_label);
            hlayout->addWidget(sfr_label);

            mem_label = new QLabel(this);
            auto vlayout = new QVBoxLayout(this);
            vlayout->addLayout(hlayout);
            vlayout->addWidget(mem_label);

            this->hide();
        }

    private:
        // labels
        QLabel* sfr_label;
        QLabel* reg_label;
        QLabel* mem_label;
        QLabel* flag_label;

    public:
        void UpdateReg(const std::vector<uint8_t>& reg_vec)
        {
            std::stringstream content;
            for (size_t reg = 0; reg < reg_vec.size(); ++reg)
                content << std::setw(5) << ("R" + std::to_string(reg) + " : ") << "0x" << std::setfill('0') << std::setw(2) << std::hex << static_cast<uint16_t>(reg_vec[reg]) << "\n";
            reg_label->setText(QString::fromStdString(content.str()));
        }

        void UpdateSFR(const std::vector<uint8_t>& sfr_vec) //
        {
            std::stringstream content;
            std::vector<const char*> reg {"PSW   : ","A     : ","B     : ","P0    : ", "P1    : ", "P2    : ", "P3    : "};
            for (size_t sfr = 0; sfr < sfr_vec.size(); ++sfr)
                content << reg.at(sfr) << "0x" << std::setfill('0') << std::setw(2) << std::hex << static_cast<uint16_t>(sfr_vec[sfr]) << '\n';

            sfr_label->setText(QString::fromStdString(content.str()));

            // update flags here
            std::vector<const char *> flags_name{
                "C   : ", "AC  : ", "F0  : ",  "RS1 : ",
                "RS0 : ", "OV  : ",  " _  : ", "P   : "};

            content.clear();
            content.str("");

            auto psw = sfr_vec[0];
            for (int bit = 7; bit >=0 ; --bit)
            {
                content << flags_name[7-bit] << std::setw(1) << (((1 << bit) & psw) > 0) << '\n';
            }
            flag_label->setText(QString::fromStdString(content.str()));
        }

        void UpdateMem(const std::vector<uint8_t> &memory) {
          // Print all 128 bytes of memory, leave the rest bytes
          // Lets make it simple
          // Me no designer and isn't good at it
          std::stringstream contents;
          contents << "Addr\n";
          for (int i = 0; i < 16; ++i) {
            contents << std::setfill('0') << std::setw(2) << std::hex << i * 8
                     << "   ->  ";
            for (int j = 0; j < 8; ++j)
              contents << std::setw(2) << std::hex
                       << static_cast<uint16_t>(memory[i * 8 + j]) << "  ";
            contents << '\n';
          }
          mem_label->setText(QString::fromStdString(contents.str()));
        }
};

#endif // ASM8051_H
