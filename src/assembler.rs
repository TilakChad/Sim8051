use std::{fs::File, io::Read};

use crate::{
    lexer::{self, Tokenizer},
    Sim8051::{self, InternalMemory},
};

// Here we will have fetch decode and the execute cycle
// We will continue parser here too .. Since regex is finished, need to device own parser

//  start   :-    ^   program   $
//
//  program :-    stmt
//          |     label stmt
//          |     e
//
//  stmt    :-    comma id sstmt
//          |     e
//
//  sstmt   :-    comma id tstmt
//          |     e
//
//  tstmt   :-    comma id
//          |     e
//
//  This grammar should work for parsing the assembler .. parsing comment isn't considered
//
//
//

use std::collections::HashMap; // or map? either way duplicated entries are to be discarded or warned

pub struct Assembler {
    pub simulator: Sim8051::Sim8051,
    pub tokenizer: lexer::Tokenizer,
    pub jmptable: HashMap<String, usize>, // represents label and their position on the source file for quick jumping
}

impl Default for Assembler {
    fn default() -> Assembler {
        Assembler {
            simulator: Sim8051::Sim8051::default(),
            tokenizer: lexer::Tokenizer::default(),
            jmptable: HashMap::new(),
        }
    }
}

impl Assembler {
    // This is the beginning of the parser
    pub fn read_src(&mut self, srcfile: String) -> bool {
        let mut src = File::open(srcfile);
        match src {
            Ok(mut file) => {
                file.read_to_string(&mut self.tokenizer.src)
                    .expect("Failed to read the open file");
                true
            }
            Err(_) => false,
        }
    }

    pub fn show_jmptable(&self) {
        println!("-------------------------------------------------- Showing jmp table of assembler ----------------------------------------");
        for (name, pos) in &self.jmptable {
            println!("{:<15} -> {:<10}", name, pos);
        }
    }

    pub fn collect_labels(&mut self) {
        self.tokenizer.pos = 1;
        let mut token;

        loop {
            token = self.tokenizer.parse_all_as_id();
            match token {
                Some(tok) => {
                    if let lexer::TokenType::ID(val) = &tok.token {
                        if val.contains(':') {
                            let labelname: String = val.chars().take_while(|x| *x != ':').collect();
                            self.jmptable.insert(labelname, self.tokenizer.pos);
                        }
                    }
                    self.tokenizer.pos += tok.len;
                }
                None => {
                    if self.tokenizer.src.chars().skip(self.tokenizer.pos).next() == Some(',') {
                        self.tokenizer.pos += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        self.tokenizer.pos = 0;
    }

    pub fn start(&mut self) {
        self.collect_labels();
        let mut ptr = self.tokenizer.src.chars();
        // do fist pass to collect all the labels
        if let Some(ch) = ptr.next() {
            if ch != '^' {
                println!("Unexpected character at the beginning of the file");
                return;
            }
        } else {
            return;
        }
        self.tokenizer.pos += 1;
        self.prog();
    }

    // This function is the main core of the parser
    // I guess LL(1) grammar should work fine
    fn prog(&mut self) {
        // It can be either stmt or label with stmt or empty
        let mut ptr = self
            .tokenizer
            .src
            .chars()
            .skip(self.tokenizer.pos)
            .skip_while(|x| x.is_ascii_whitespace());
        if let Some(ch) = ptr.next() {
            if ch == '$' {
                return;
            }
        }
        // we should be parsing the string here and returns token as id or string of chracter.. not as actual token type due to various addressing modes and their different semantics
        let token = self.tokenizer.parse_all_as_id();
        // differentiate this token as label or as id
        match token {
            None => println!("Unexpected token found here"),

            Some(tok) => {
                if let lexer::TokenType::ID(val) = &tok.token {
                    if val.contains(':') {
                        self.tokenizer.pos += tok.len;
                        // Read another token now
                        let newtoken = self.tokenizer.parse_all_as_id();
                        if let Some(z) = newtoken {
                            if self.stmt(&z) {
                                self.prog()
                            };
                        } else {
                            println!("Invalid token here : ");
                        }
                    } else {
                        // It has to be id
                        if self.stmt(&tok) {
                            self.prog();
                        }
                    }
                }
            }
        }
    }

    fn stmt(&mut self, prev_token: &lexer::Token) -> bool {
        // This token better be
        // This func now should decide to return or call the second statement
        // This resulting token better be a asm instruction
        // Lets differ match, lets just try parsing
        self.tokenizer.pos += prev_token.len;

        // else its the single instruction command .. execute it right here
        // use ad hoc context sensitiveness here

        if let lexer::TokenType::ID(ins) = &prev_token.token {
            let pass = match ins.as_str() {
                // list all single instructions here
                "ret" => {
                    let returnpos = self.jmptable.get(&String::from("ret"));
                    if let Some(&pos) = returnpos {
                        self.jmptable.remove(&String::from("ret"));
                        self.tokenizer.pos = pos;
                    }
                    println!("Found ret command");
                    true
                }
                "nop" => {
                    // Does nothing
                    true
                }
                "end" => {
                    println!("\n\nReached at the end of the program\n");
                    false
                }
                _ => {
                    // Move toward another branch
                    self.sstmt(&ins)
                }
            };
            pass
        } else {
            println!("Unexpected token found here in stmt branch...");
            false
        }
    }

    fn sstmt(&mut self, command: &str) -> bool {
        // This function carries the previous command and is ready to parse the next operand
        // Now get the first argument from here
        let next_token = self.tokenizer.parse_all_as_id();
        // It better be a id
        match next_token {
            None => false,
            Some(token) => {
                // It can be anything now .. R0, @R0, 22H, P0.1 like these
                if let lexer::TokenType::ID(ins) = &token.token {
                    self.tokenizer.pos += token.len;
                    // if it successfully passed get ready for another statement
                    if self.tokenizer.consume_comma() {
                        self.tstmt(command, ins)
                    } else {
                        // check for single instruction command here
                        match command {
                            // since clr,  setb and cpl are quite similar, they can be merged
                            "clr" => clr_set_cpl(
                                self,
                                command,
                                ins.as_str(),
                                InternalMemory::reset_bit_addressable,
                            ),
                            "cpl" => clr_set_cpl(
                                self,
                                command,
                                ins.as_str(),
                                InternalMemory::complement_bit_addressable,
                            ),
                            "setb" => clr_set_cpl(
                                self,
                                command,
                                ins.as_str(),
                                InternalMemory::set_bit_addressable,
                            ),
                            "swap" => {
                                match ins.as_str() {
                                    "A" => {
                                        // Swap Nibbles of A the accumulator
                                        let loc =
                                            Sim8051::sfr_addr(&self.simulator.accumulator) as usize;
                                        let val = self.simulator.internal_memory.memory[loc];
                                        self.simulator.internal_memory.memory[loc] =
                                            ((val & 0x0F) << 4) | ((val & 0xF0) >> 4);
                                        true
                                    }
                                    _ => {
                                        println!("Invalid argument to swap instruction .. Expected accumulator");
                                        false
                                    }
                                }
                            }
                            "rl" => {
                                match ins.as_str() {
                                    "A" => {
                                        // Naive implementation
                                        let acc = Sim8051::sfr_addr(&self.simulator.accumulator);
                                        let val =
                                            self.simulator.internal_memory.memory[acc as usize];
                                        self.simulator.internal_memory.memory[acc as usize] =
                                            ((val & 0x80) >> 7) | (val << 1);
                                        true
                                    }
                                    _ => {
                                        println!("Invalid operand to rl");
                                        false
                                    }
                                }
                            }
                            "rr" => {
                                match ins.as_str() {
                                    "A" => {
                                        // TODO
                                        let acc = Sim8051::sfr_addr(&self.simulator.accumulator);
                                        let val =
                                            self.simulator.internal_memory.memory[acc as usize];
                                        self.simulator.internal_memory.memory[acc as usize] =
                                            ((val & 0x01) << 7) | (val >> 1);
                                        true
                                    }
                                    _ => {
                                        println!("Invalid argument to rr");
                                        false
                                    }
                                }
                            }
                            "rlc" => {
                                match ins.as_str() {
                                    "A" => {
                                        // Get the content of the carry flag first
                                        let psw = Sim8051::sfr_addr(&self.simulator.psw);
                                        let cflag = (self.simulator.internal_memory.memory
                                            [psw as usize]
                                            & 0x80)
                                            >> 7;

                                        let acc = Sim8051::sfr_addr(&self.simulator.accumulator);
                                        let val =
                                            self.simulator.internal_memory.memory[acc as usize];

                                        let acc_msb = val >> 7;
                                        if acc_msb == 1 {
                                            self.simulator.internal_memory.memory[psw as usize] |=
                                                0x80;
                                        } else {
                                            self.simulator.internal_memory.memory[psw as usize] &=
                                                0x7F;
                                        }
                                        self.simulator.internal_memory.memory[acc as usize] =
                                            (val << 1) | cflag;
                                        true
                                    }
                                    _ => {
                                        println!("Invalid argument to rr");
                                        false
                                    }
                                }
                            }
                            "rrc" => {
                                match ins.as_str() {
                                    "A" => {
                                        // Get the content of the carry flag first
                                        let psw = Sim8051::sfr_addr(&self.simulator.psw);
                                        let cflag = (self.simulator.internal_memory.memory
                                            [psw as usize]
                                            & 0x80)
                                            >> 7;

                                        let acc = Sim8051::sfr_addr(&self.simulator.accumulator);
                                        let val =
                                            self.simulator.internal_memory.memory[acc as usize];

                                        let acc_lsb = val & 0x01;
                                        if acc_lsb == 1 {
                                            self.simulator.internal_memory.memory[psw as usize] |=
                                                0x80;
                                        } else {
                                            self.simulator.internal_memory.memory[psw as usize] &=
                                                0x7F;
                                        }
                                        self.simulator.internal_memory.memory[acc as usize] =
                                            (val >> 1) | (cflag << 7);
                                        true
                                    }
                                    _ => {
                                        println!("Invalid argument to rr");
                                        false
                                    }
                                }
                            }
                            "jc" | "jnc" | "jz" | "jnz" => {
                                let pos = self.jmptable.get(ins);
                                if let Some(&val) = &pos {
                                    // TODO :: Check for the carry bit and jump here
                                    let jmp_condition = if command == "jc" || command == "jnc" {
                                        // check carry bit '
                                        let carry_set = (self.simulator.internal_memory.memory
                                            [Sim8051::sfr_addr(&self.simulator.psw) as usize]
                                            & 0x80)
                                            > 0;
                                        if carry_set && command == "jc" {
                                            true
                                        } else if !carry_set && command == "jnc" {
                                            true
                                        } else {
                                            false
                                        }
                                    } else {
                                        let is_acc_zero = (self.simulator.internal_memory.memory
                                            [Sim8051::sfr_addr(&self.simulator.accumulator)
                                                as usize])
                                            == 0;
                                        if is_acc_zero && command == "jz" {
                                            true
                                        } else if !is_acc_zero && command == "jnz" {
                                            true
                                        } else {
                                            false
                                        }
                                    };
                                    if jmp_condition {
                                        self.tokenizer.pos = val;
                                    };
                                    true
                                } else {
                                    println!("Invalid label found {}", ins);
                                    false
                                }
                            }
                            // haven't explored pattern binding in or pattern
                            // TODO :: DPTR for inc
                            inst @ "dec" | inst @ "inc" => {
                                let step = if inst == "dec" { -1 } else { 1 };
                                // go through all the addressing modes ..
                                // continuing further in the belief of rust using 2's complement for their negative number
                                let memloc = if ins == "A" {
                                    Some(Sim8051::sfr_addr(&self.simulator.accumulator))
                                } else {
                                    if let Some(operand) = lexer::Tokenizer::parse_all(ins.as_str())
                                    {
                                        use lexer::TokenType::*;
                                        match operand.token {
                                            IMM(_) => {
                                                println!(
                                                    "Cannot use immediate value as operand to {}",
                                                    inst
                                                );
                                                None
                                            }
                                            HEX(loc) => Some(loc as u8),
                                            ID(reg) => {
                                                use std::str::FromStr;
                                                // Direct register location
                                                let reg = Sim8051::ScratchpadRegisters::from_str(
                                                    reg.as_str(),
                                                )
                                                .expect("Not a scratchpad register.. error");
                                                // Return its location depending upon the currently selected register bank
                                                let pswloc = Sim8051::sfr_addr(&Sim8051::SFR::Reg(
                                                    Sim8051::IRegs::PSW,
                                                ))
                                                    as usize;
                                                let count: u8 = (0x18
                                                    & self.simulator.internal_memory.memory
                                                        [pswloc])
                                                    >> 3;
                                                Some(count * 8 + reg.reg_count())
                                            }
                                            IND(reg) => {
                                                let pswloc = Sim8051::sfr_addr(&Sim8051::SFR::Reg(
                                                    Sim8051::IRegs::PSW,
                                                ))
                                                    as usize;
                                                let count: u8 = (0x18
                                                    & self.simulator.internal_memory.memory
                                                        [pswloc])
                                                    >> 3;
                                                Some(
                                                    self.simulator.internal_memory.memory
                                                        [(count * 8 + reg.reg_count()) as usize],
                                                )
                                            }
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    }
                                };
                                // get the content of that memory location as i16 first and then do some casting and manipulation here and there
                                let val = self.simulator.internal_memory.memory
                                    [memloc.unwrap() as usize]
                                    as i16;
                                let ans = ((val + step) & 0xFF) as u8;
                                self.simulator.internal_memory.memory[memloc.unwrap() as usize] =
                                    ans;
                                true
                            }
                            ch @ "ajmp" | ch @ "acall" => {
                                // just jmp to the label from here
                                // How should it change the syntax parsing process?
                                let label = String::from(ins);
                                let pos = self.jmptable.get(&label);
                                if let Some(&y) = pos {
                                    if ch == "ajmp" {
                                        self.tokenizer.pos = y;
                                    } else {
                                        // push current position for return statement
                                        // using ret .. too bored to change datastructure
                                        self.jmptable
                                            .insert(String::from("ret"), self.tokenizer.pos);
                                        self.tokenizer.pos = y;
                                    }
                                    true
                                } else {
                                    println!("\nInvalid {} command : {} not found.", ch, label);
                                    true
                                }
                            }
                            ch @ "push" | ch @ "pop" => {
                                // The addressing modes of the push and pop operations aren't cleared from Keil documentation
                                // TODO:: So, we only allowing registers and accumulator to be pushed into for now
                                //
                                if let Some(op) = lexer::Tokenizer::parse_all(ins) {
                                    use lexer::TokenType::*;
                                    let src_addr = match op.token {
                                        HEX(hex) => Some(hex as u8),
                                        ID(id) => {
                                            if id == "A" {
                                                Some(Sim8051::sfr_addr(&self.simulator.accumulator))
                                            } else if id == "B" {
                                                Some(Sim8051::sfr_addr(&Sim8051::SFR::Reg(
                                                    Sim8051::IRegs::B,
                                                )))
                                            } else {
                                                use std::str::FromStr;
                                                match Sim8051::ScratchpadRegisters::from_str(&id) {
                                                    Ok(reg) => {
                                                        let count: u8 = (0x18
                                                            & self
                                                                .simulator
                                                                .internal_memory
                                                                .memory
                                                                [Sim8051::sfr_addr(
                                                                    &self.simulator.psw,
                                                                )
                                                                    as usize])
                                                            >> 3;
                                                        Some(count * 8 + reg.reg_count())
                                                    }
                                                    Err(_) => {
                                                        //
                                                        println!(
                                                            "Invalid operand to {} -> {}.",
                                                            ins, ch
                                                        );
                                                        None
                                                    }
                                                }
                                            }
                                        }
                                        _ => None,
                                    };
                                    // if it is push, retrieve its content
                                    let sp_loc =
                                        Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::SP))
                                            as usize;
                                    let sp_val = self.simulator.internal_memory.memory[sp_loc];
                                    if ch == "push" {
                                        // cannot use reference to that memory due to borro
                                        self.simulator.internal_memory.memory[sp_loc] += 1;
                                        self.simulator.internal_memory.memory[self
                                            .simulator
                                            .internal_memory
                                            .memory[sp_loc]
                                            as usize] = self.simulator.internal_memory.memory
                                            [src_addr.unwrap() as usize];
                                    } else {
                                        self.simulator.internal_memory.memory
                                            [src_addr.unwrap() as usize] =
                                            self.simulator.internal_memory.memory[self
                                                .simulator
                                                .internal_memory
                                                .memory[sp_loc]
                                                as usize];
                                        self.simulator.internal_memory.memory[sp_loc] -= 1;
                                    }
                                    true
                                } else {
                                    println!("Invalid operand {} to {}.", ins, ch);
                                    false
                                }
                            }

                            ch @ "mul" | ch @ "div" => {
                                if ins == "AB" {
                                    let op1_addr =
                                        Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::B));
                                    let op2_addr = Sim8051::sfr_addr(&self.simulator.accumulator);
                                    let psw_loc = Sim8051::sfr_addr(&self.simulator.psw);
                                    // Reset the carry flag
                                    self.simulator.internal_memory.memory[psw_loc as usize] &= 0x7F;
                                    // Reset the overflow flag
                                    self.simulator.internal_memory.memory[psw_loc as usize] &= 0xFB;
                                    if ch == "mul" {
                                        let product: u16 = self.simulator.internal_memory.memory
                                            [op1_addr as usize]
                                            as u16
                                            * self.simulator.internal_memory.memory
                                                [op2_addr as usize]
                                                as u16;
                                        if (product & 0xFF00) > 1 {
                                            // set overflow flag
                                            self.simulator.internal_memory.memory
                                                [psw_loc as usize] |= 0x04;
                                        }

                                        self.simulator.internal_memory.memory[op2_addr as usize] =
                                            (product & 0x00FF) as u8;
                                        self.simulator.internal_memory.memory[op1_addr as usize] =
                                            ((product & 0xFF00) >> 8) as u8;
                                        // Its not specified whose parity bit is taken.. Assuming its the accumulator's
                                        self.simulator.set_parity_bit(
                                            self.simulator.internal_memory.memory
                                                [op1_addr as usize],
                                        );
                                    } else {
                                        let a = self.simulator.internal_memory.memory
                                            [op2_addr as usize];
                                        let b = self.simulator.internal_memory.memory
                                            [op1_addr as usize];

                                        if b == 0x00 {
                                            self.simulator.internal_memory.memory
                                                [psw_loc as usize] |= 0x04;
                                            println!("Warning : Division by zero attempted");
                                        } else {
                                            self.simulator.internal_memory.memory
                                                [op2_addr as usize] = a / b;
                                            self.simulator.internal_memory.memory
                                                [op1_addr as usize] = a % b;
                                        }
                                    }
                                    true
                                } else {
                                    println!("Invalid operand : {} to mul.", ins);
                                    false
                                }
                            }
                            _ => {
                                // This not a single operand instruction
                                println!("Invalid single operand instruction.");
                                false
                            }
                        }
                    }
                } else {
                    println!("No next token found.. -_-");
                    false
                }
            }
        }
    }

    fn tstmt(&mut self, command: &str, first: &str) -> bool {
        // Most of the instructions fall on this category
        // Retrieve the second operand here
        // Lets start with basic move instruction which everyone loves
        let next_token = self.tokenizer.parse_all_as_id();

        let mut success = true;

        if let Some(token) = next_token {
            if let lexer::TokenType::ID(second) = token.token {
                if let (Some(op1), Some(op2)) = (
                    lexer::Tokenizer::parse_all(first),
                    lexer::Tokenizer::parse_all(&second),
                ) {
                    self.tokenizer.pos += token.len;
                    if self.tokenizer.consume_comma() {
                        println!("Consumed comma and getting ready");
                        self.fstmt(command, &first, &second);
                    } else {
                        // Now execute the command
                        use lexer::TokenType::*;
                        use std::str::FromStr;

                        match command {
                            "mov" => {
                                let src = match op1.token {
                                    HEX(hex) => Some(hex as u8),
                                    IMM(_) => None,
                                    ID(reg) => {
                                        // This is the direct register addressing mode .. it should be a register

                                        // Return its location depending upon the currently selected register bank
                                        let pswloc = Sim8051::sfr_addr(&Sim8051::SFR::Reg(
                                            Sim8051::IRegs::PSW,
                                        ))
                                            as usize;
                                        let count = (0x18
                                            & self.simulator.internal_memory.memory[pswloc])
                                            >> 3;
                                        let start = count * 8;

                                        // try parsing it as a scratchpad register
                                        // or it could be a port too
                                        // TODO :: PSW is directly writable from here, so maybe consider changing the behaviour
                                        let memloc = match Sim8051::ScratchpadRegisters::from_str(
                                            reg.as_str(),
                                        ) {
                                            Ok(reg) => start + reg.reg_count(),
                                            Err(_) => {
                                                // before that see it it is A
                                                if reg == "A" {
                                                    Sim8051::sfr_addr(&self.simulator.accumulator)
                                                }
                                                // else if reg == "SP" {
                                                //     Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::SP))
                                                // }
                                                else {
                                                    Sim8051::sfr_addr(
                                                        &Sim8051::SFR::from_str(reg.as_str())
                                                            .expect(
                                                            "Not a scratchpad or port or Acc or B",
                                                        ),
                                                    )
                                                }
                                            }
                                        };

                                        Some(memloc)
                                    }
                                    // For indirect addressing, retrieve the value of the register to use as src location
                                    IND(reg) => {
                                        let pswloc = Sim8051::sfr_addr(&Sim8051::SFR::Reg(
                                            Sim8051::IRegs::PSW,
                                        ))
                                            as usize;
                                        let count = (0x18
                                            & self.simulator.internal_memory.memory[pswloc])
                                            >> 3;
                                        let val = count * 8 + reg.reg_count();
                                        Some(self.simulator.internal_memory.memory[val as usize])
                                    }
                                    _ => None,
                                };
                                let dest = match op2.token {
                                    HEX(hex) => {
                                        Some(self.simulator.internal_memory.memory[hex as usize])
                                    }
                                    IMM(hex) => Some(hex as u8), // This is the error but can't return anything here .. so changing the return type
                                    ID(reg) => {
                                        let pswloc = Sim8051::sfr_addr(&Sim8051::SFR::Reg(
                                            Sim8051::IRegs::PSW,
                                        ))
                                            as usize;
                                        let count = (0x18
                                            & self.simulator.internal_memory.memory[pswloc])
                                            >> 3;
                                        let start = count * 8;

                                        let memloc = match Sim8051::ScratchpadRegisters::from_str(
                                            reg.as_str(),
                                        ) {
                                            Ok(reg) => start + reg.reg_count(),
                                            Err(_) => {
                                                if reg == "A" {
                                                    Sim8051::sfr_addr(&self.simulator.accumulator)
                                                } else {
                                                    Sim8051::sfr_addr(
                                                        &Sim8051::SFR::from_str(reg.as_str())
                                                            .expect(
                                                            "Not a scratchpad or port or Acc or B",
                                                        ),
                                                    )
                                                }
                                            }
                                        };
                                        Some(self.simulator.internal_memory.memory[memloc as usize])
                                    }
                                    // For indirect addressing, retrieve the value of the register to use as src location
                                    IND(reg) => {
                                        let pswloc = Sim8051::sfr_addr(&Sim8051::SFR::Reg(
                                            Sim8051::IRegs::PSW,
                                        ))
                                            as usize;
                                        let count = (0x18
                                            & self.simulator.internal_memory.memory[pswloc])
                                            >> 3;
                                        let val = count * 8 + reg.reg_count();
                                        Some(
                                            self.simulator.internal_memory.memory[self
                                                .simulator
                                                .internal_memory
                                                .memory
                                                [val as usize]
                                                as usize],
                                        )
                                    }
                                    _ => None,
                                };
                                println!("Moved from {} to {}", src.unwrap(), dest.unwrap());
                                self.simulator.mov(src.unwrap(), dest.unwrap());
                                success = true;
                            }
                            // where's the single binding?
                            inst @ "add" | inst @ "addc" | inst @ "subb" => {
                                // First operand is always A .. makes one thing easy
                                // lol .. need to update flags accordingly now
                                // There are four main flags that need to be set :
                                // C, AC, OV and P :- Getting boooorring now
                                // TODO :: Correct AC flag for subb instruction .. Who would care for that though .. but still
                                if first == "A" {
                                    let operand =
                                        lexer::retrieve_rvalue(&mut self.simulator, &op2.token);
                                    // Allow addition with wraparound effect
                                    let addr = Sim8051::sfr_addr(&self.simulator.accumulator);
                                    let mut val =
                                        self.simulator.internal_memory.memory[addr as usize] as i32;
                                    let mut should_set_carry = false;
                                    let mut factor = 1;
                                    if inst == "addc" || inst == "subb" {
                                        // check the carry flag
                                        // special case if 0xff is A
                                        if inst == "subb" {
                                            factor = -1;
                                        }
                                        let psw = Sim8051::sfr_addr(&self.simulator.psw);
                                        if (psw & 0x80) > 0 {
                                            val += factor;
                                            if val & 0xff00 > 1 {
                                                should_set_carry = true;
                                            }
                                            val = val & 0xff;
                                        }
                                    }

                                    let temp = val;
                                    val = val + factor * operand.unwrap() as i32;
                                    let ans = (val & 0xff) as u8;
                                    self.simulator.internal_memory.memory[addr as usize] = ans;
                                    // setting these flags is plain pain
                                    use std::ops::BitOr;
                                    should_set_carry =
                                        should_set_carry.bitor((val as i32 & 0xff00) > 0);
                                    self.simulator.set_carry_bit(should_set_carry);
                                    self.simulator.set_parity_bit(ans);

                                    let should_set_parity =
                                        ((temp & 0x000F + operand.unwrap() as i32 & 0x000F) >> 4)
                                            > 0;
                                    self.simulator.set_auxiliary_carry_bit(should_set_parity);

                                    // lastly overflow bit
                                    // overflow bit is set when there's overflow from 7th bit or 8th bit but not from both
                                    let is_carry_to_msb =
                                        ((temp & 0x007F) + (operand.unwrap() as i32 & 0x007F) >> 7)
                                            > 0;
                                    use std::ops::BitXor;
                                    self.simulator.set_auxiliary_carry_bit(
                                        should_set_carry.bitxor(is_carry_to_msb),
                                    );
                                } else {
                                    println!("Invalid operand to {} instruction ", inst);
                                    success = false;
                                }
                            }
                            // "subb" => { // I guess subb can be merged with addc and add instructions // merged above
                            // Implementing these bitwise operations instruction is a different kind of pain
                            // They have 8 addressing modes .. -_- -_- -_-
                            "anl" => anl_orl_xrl(self, command, first, &second, |x, y| {
                                use std::ops::BitAnd;
                                x.bitand(y)
                            }),
                            "orl" => anl_orl_xrl(self, command, first, &second, |x, y| {
                                use std::ops::BitOr;
                                x.bitor(y)
                            }),
                            "xrl" => anl_orl_xrl(self, command, first, &second, |x, y| {
                                use std::ops::BitXor;
                                x.bitxor(y)
                            }),

                            "jb" | "jnb" => {
                                // Either in a bit addressable location or bit addressable register
                                if let Some(token) = lexer::Tokenizer::parse_all(first) {
                                    use lexer::TokenType::*;
                                    let condition = match token.token {
                                        HEX(val) => self
                                            .simulator
                                            .internal_memory
                                            .get_bit_status_addr_memory(val as u8),
                                        BIT_ADDR(reg, bit) => {
                                            // Retrieve operand manually
                                            (self.simulator.internal_memory.memory
                                                [Sim8051::sfr_addr(&reg) as usize]
                                                & (1 << bit))
                                                > 0
                                        }
                                        _ => {
                                            panic!(
                                            "Panicking once again for not being bit addressable"
                                        );
                                        }
                                    };
                                    // parse the label
                                    let pos = self.jmptable.get(&second);
                                    if let Some(&val) = &pos {
                                        if condition && command == "jb" {
                                            self.tokenizer.pos = val;
                                        } else if !condition && command == "jnb" {
                                            self.tokenizer.pos = val;
                                        }
                                    } else {
                                        success = false;
                                        panic!("Not a valid jmp label {}", second);
                                    }
                                } else {
                                    println!("Not a bit addressable value panic");
                                    success = false;
                                }
                            }
                            "djnz" => {
                                // what does this command do ?
                                // -> Decrease byte at given address and jmp to label if it is not zero
                                // only allows direct and register addressing mode
                                // Doesn't know what keil does for direct addressing/register addressing .. if it allows Port address or not .. NO keil installed
                                if let Some(to) = lexer::Tokenizer::parse_all(first) {
                                    use lexer::TokenType::*;
                                    let addr = match to.token {
                                        HEX(hex) => Some(hex as u8),
                                        ID(id) => {
                                            // parse as sctrachpad register
                                            // locate current register bank first
                                            let reg = Sim8051::ScratchpadRegisters::from_str(&id)
                                                .expect("Not a scratchpad register error ...");
                                            let pswloc = Sim8051::sfr_addr(&Sim8051::SFR::Reg(
                                                Sim8051::IRegs::PSW,
                                            ))
                                                as usize;
                                            let count: u8 = (0x18
                                                & self.simulator.internal_memory.memory[pswloc])
                                                >> 3;
                                            Some(count * 8 + reg.reg_count())
                                        }
                                        _ => {
                                            println!("Invalid addressing mode to djnz : {}", first);
                                            success = false;
                                            None
                                        }
                                    };
                                    // decrease the value at that location by 1 using wraparound arithmetic
                                    let mut refval = &mut self.simulator.internal_memory.memory
                                        [addr.unwrap() as usize];
                                    *refval = ((*refval as i16 - 1) & 0x00FF) as u8;
                                    // Now jump if it needs to
                                    let pos = self.jmptable.get(&second);
                                    if let Some(&val) = &pos {
                                        self.tokenizer.pos = val;
                                    } else {
                                        success = false;
                                        panic!("Not a valid jmp label {}", second);
                                    }
                                } else {
                                    println!("Invalid first operand to djnz : {}", first);
                                    success = false
                                }
                            }
                            _ => success = false,
                        }
                    }
                }
            }
        }
        success
    }

    // fourth statement procedure and  will handle instructions wth 3 operands
    fn fstmt(&mut self, command: &str, first: &str, second: &str) {
        let new_token = self.tokenizer.parse_all_as_id();
        let pswloc = Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::PSW)) as usize;
        let count: u8 = (0x18 & self.simulator.internal_memory.memory[pswloc]) >> 3;
        // lol.. can't use logical and with if let
        if let Some(tok) = new_token {
            // If I were to rewrite it, the addressing mode thing could have been done much more nicely
            if command == "cjne" {
                // wtf .. why cjne had to set carry flag .. didn't they find any easier way for conditional branching
                // parse the first argument
                // Its either A, Rn or @Rn
                if let (Some(op1), Some(op2)) = (
                    lexer::Tokenizer::parse_all(&first),
                    lexer::Tokenizer::parse_all(&second),
                ) {
                    use lexer::TokenType::*;
                    use std::str::FromStr;
                    let mut involve_acc = false;
                    let src_op = match op1.token {
                        IND(reg) => {
                            // Retrieve the active memory bank
                            Some(
                                self.simulator.internal_memory.memory
                                    [(count * 8 + reg.reg_count()) as usize],
                            )
                        }
                        ID(id) => {
                            if id == "A" {
                                // yo toriley feri direct addressing linxa
                                involve_acc = true;
                                Some(Sim8051::sfr_addr(&self.simulator.accumulator))
                            } else {
                                match Sim8051::ScratchpadRegisters::from_str(id.as_str()) {
                                    Ok(T) => Some((count * 8 + T.reg_count()) as u8),
                                    Err(_) => None,
                                }
                            }
                        }
                        _ => None,
                    };
                    let dest_val = match op2.token {
                        IMM(hex) => Some(hex as u8),
                        ID(id) => {
                            if involve_acc {
                                match Sim8051::ScratchpadRegisters::from_str(id.as_str()) {
                                    Ok(T) => Some(
                                        self.simulator.internal_memory.memory
                                            [(count * 8 + T.reg_count()) as usize],
                                    ),
                                    Err(_) => None,
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    // stupid instructon
                    let src_val = self.simulator.internal_memory.memory[src_op.unwrap() as usize];
                    let psw_loc = Sim8051::sfr_addr(&self.simulator.psw) as usize;
                    let refpsw = &mut self.simulator.internal_memory.memory[psw_loc];
                    if src_val < dest_val.unwrap() {
                        // set the carry flag or reset if lmao
                        *refpsw = *refpsw | 0x80;
                    } else {
                        *refpsw &= 0x7F;
                    }
                    // Prepare for long jump .. get set go
                    if let lexer::TokenType::ID(id) = tok.token {
                        let jmp_pos = self.jmptable.get(&id);
                        if let Some(&val) = jmp_pos {
                            self.tokenizer.pos = val;
                        } else {
                            println!("Invalid jmp attempted to label {}", id);
                        }
                    }
                } else {
                    println!("Invalid operands {} and {} to cjne", first, second);
                }
            } else {
                println!("Invalid three argument command {}", command);
            }
        } else {
            println!("Invald token at line {}", self.tokenizer.pos);
        }
    }
}

fn clr_set_cpl(
    asm: &mut Assembler,
    ins: &str,
    operand: &str,
    operator: fn(&mut InternalMemory, u8, u8),
) -> bool {
    if (ins == "cpl") || (ins == "clr") {
        if operand == "A" {
            // Clear the contents of the accumulator
            let loc = Sim8051::sfr_addr(&asm.simulator.accumulator);
            for i in 0..8 {
                asm.simulator
                    .internal_memory
                    .operate_bit_addressable_memory(loc + i, operator);
                return true;
            }
        }
    }
    match operand {
        "C" => {
            // Reset the carry flag
            let psw = Sim8051::SFR::Reg(Sim8051::IRegs::PSW);
            asm.simulator
                .internal_memory
                .operate_bit_addressable_registers(Sim8051::sfr_addr(&psw), 7, operator);
            true
        }
        // It also support resetting of bit addressable memory
        rstr => {
            if let Some(hex) = lexer::Tokenizer::parse_hex(rstr) {
                asm.simulator
                    .internal_memory
                    .operate_bit_addressable_memory(hex as u8, operator);
                true
            } else {
                // Try to parse it as bit addressable registers
                if let Some(bitaddr) = Tokenizer::parse_bitaddr(rstr) {
                    match bitaddr.token {
                        lexer::TokenType::BIT_ADDR(sfr, bit) => {
                            asm.simulator
                                .internal_memory
                                .operate_bit_addressable_registers(
                                    Sim8051::sfr_addr(&sfr),
                                    bit,
                                    operator,
                                );
                            true
                        }
                        _ => false,
                    }
                } else {
                    println!("Error not a bit address");
                    false
                }
            }
        }
    }
}

fn anl_orl_xrl(
    asm: &mut Assembler,
    ins: &str,
    op1: &str,
    op2: &str,
    operator: fn(bool, bool) -> bool,
) {
    // let's go 8 addressing modes.. tf
    // first with the single operations on the carry bit
    match op1 {
        "C" => {
            // There's this stupid syntax that came from nowhere just for these logical instructions
            // ORL C, /22h
            // pattern matching not working.. not strong as Haskell's
            let pswloc = Sim8051::sfr_addr(&asm.simulator.psw);
            if op2.starts_with("/") {
                // parse remaining string as simple hex
                if let Some(hex) = lexer::Tokenizer::parse_hex(&op2[1..]) {
                    // Retrieve bitwise value at that bit addressable location .. .jhyau
                    let operand = !asm
                        .simulator
                        .internal_memory
                        .get_bit_status_addr_memory(hex as u8);

                    let result = operator(operand, (pswloc & 0x80) > 0);
                    if result {
                        asm.simulator.internal_memory.memory[pswloc as usize] |= 0x80;
                    } else {
                        asm.simulator.internal_memory.memory[pswloc as usize] &= 0x7F;
                    }
                } else {
                    panic!("Look who is giving garbage after {} instruction", ins);
                }
            } else {
                // TODO :: Reduce these repeated codes simply using Option<T>
                if let Some(token) = lexer::Tokenizer::parse_all(op2) {
                    use lexer::TokenType::*;
                    match token.token {
                        HEX(val) => {
                            let operand = asm
                                .simulator
                                .internal_memory
                                .get_bit_status_addr_memory(val as u8);
                            // Gonna copy paste
                            let result = operator(operand, (pswloc & 0x80) > 0);
                            if result {
                                asm.simulator.internal_memory.memory[pswloc as usize] |= 0x80;
                            } else {
                                asm.simulator.internal_memory.memory[pswloc as usize] &= 0x7F;
                            }
                        }
                        BIT_ADDR(reg, bit) => {
                            // Retrieve operand manually
                            let operand = (asm.simulator.internal_memory.memory
                                [Sim8051::sfr_addr(&reg) as usize]
                                & (1 << bit))
                                > 0;
                            let result = operator(operand, (pswloc & 0x80) > 0);
                            if result {
                                asm.simulator.internal_memory.memory[pswloc as usize] |= 0x80;
                            } else {
                                asm.simulator.internal_memory.memory[pswloc as usize] &= 0x7F;
                            }
                        }
                        _ => {
                            panic!("Panicking once again")
                        }
                    }
                } else {
                    panic!("Invalid token to {}", op2);
                }
            }
        }
        // then on accumulator
        "A" => {
            // Takes every addressing mode .. so copy paste
            if let Some(op2token) = lexer::Tokenizer::parse_all(op2) {
                use lexer::TokenType::*;
                let val = match op2token.token {
                    HEX(hex) => Some(asm.simulator.internal_memory.memory[hex as usize]),
                    IMM(hex) => Some(hex as u8), // This is the error but can't return anything here .. so changing the return type
                    ID(reg) => {
                        let pswloc =
                            Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::PSW)) as usize;
                        let count = (0x18 & asm.simulator.internal_memory.memory[pswloc]) >> 3;
                        let start = count * 8;

                        use std::str::FromStr;
                        let memloc = match Sim8051::ScratchpadRegisters::from_str(reg.as_str()) {
                            Ok(reg) => start + reg.reg_count(),
                            Err(_) => {
                                // try parsing as port address now
                                let port = Sim8051::sfr_addr(
                                    &Sim8051::SFR::from_str(reg.as_str())
                                        .expect("Not a scratchpad or port or Acc or B"),
                                );
                                port
                            }
                        };
                        Some(asm.simulator.internal_memory.memory[memloc as usize])
                    }
                    // For indirect addressing, retrieve the value of the register to use as src location
                    IND(reg) => {
                        let pswloc =
                            Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::PSW)) as usize;
                        let count = (0x18 & asm.simulator.internal_memory.memory[pswloc]) >> 3;
                        let val = count * 8 + reg.reg_count();
                        Some(
                            asm.simulator.internal_memory.memory
                                [asm.simulator.internal_memory.memory[val as usize] as usize],
                        )
                    }
                    _ => None,
                };
                // trying to unwrap here whitout further checking
                let accloc = Sim8051::sfr_addr(&asm.simulator.accumulator);
                let mut acc = asm.simulator.internal_memory.memory[accloc as usize];
                let unwrapped = val.unwrap();
                let mut abit;
                let mut bbit;
                for i in 0..8 {
                    abit = (acc & (1 << i)) > 0;
                    bbit = (unwrapped & (1 << i)) > 0;
                    let result = operator(abit, bbit);
                    if result {
                        acc |= 1 << i;
                    } else {
                        acc &= !(1 << i);
                    }
                }
                asm.simulator.internal_memory.memory[accloc as usize] = acc;
            }
        }
        _ => {
            panic!("Not handling those last two cases");
        } // and finally on direct addressing mode
    }
}
