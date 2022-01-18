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

pub struct Assembler {
    pub simulator: Sim8051::Sim8051,
    pub tokenizer: lexer::Tokenizer,
}

impl Default for Assembler {
    fn default() -> Assembler {
        Assembler {
            simulator: Sim8051::Sim8051::default(),
            tokenizer: lexer::Tokenizer::default(),
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

    pub fn start(&mut self) {
        let mut ptr = self.tokenizer.src.chars();
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
                println!("End of the parser reached");
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
                        println!("Expanded along the grammar => label stmt prog.");
                        self.tokenizer.pos += tok.len;

                        // Read another token now
                        let newtoken = self.tokenizer.parse_all_as_id();
                        if let Some(z) = newtoken {
                            self.stmt(&z);
                            self.prog();
                        } else {
                            println!("Invalid token here : ");
                        }
                    } else {
                        // It has to be id
                        self.stmt(&tok);
                        self.prog();
                    }
                }
            }
        }
    }

    fn stmt(&mut self, prev_token: &lexer::Token) {
        // This token better be
        // This func now should decide to return or call the second statement
        // This resulting token better be a asm instruction
        // Lets differ match, lets just try parsing
        println!("Expanded along : stmt -> stmt prog");
        self.tokenizer.pos += prev_token.len;

        // else its the single instruction command .. execute it right here
        // use ad hoc context sensitiveness here

        if let lexer::TokenType::ID(ins) = &prev_token.token {
            match ins.as_str() {
                // list all single instructions here
                "ret" => println!("Found ret command"),
                "swapa" => println!("Found swap command"),
                "rla" => println!("Found rl command"),
                _ => {
                    // Move toward another branch
                    self.sstmt(&ins);
                }
            }
            self.prog();
        } else {
            println!("Unexpected token found here in stmt branch...");
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
                                        // TODO :: Implement this
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

                            "jb" => true,
                            "jnb" => true,
                            "jbc" => true,

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
                                                Some(count * 8 + reg.reg_count())
                                            }
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    }
                                };
                                // get the content of that memory location as i16 first and then do some casting and manipulation here and there
                                let val  = self.simulator.internal_memory.memory[memloc.unwrap() as usize] as i16;
                                let ans  = ((val + step) & 0xFF) as u8;
                                self.simulator.internal_memory.memory[memloc.unwrap() as usize] = ans;
                                true
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

        let mut success = false;

        if let Some(token) = next_token {
            if let lexer::TokenType::ID(second) = token.token {
                if let (Some(op1), Some(op2)) = (
                    lexer::Tokenizer::parse_all(first),
                    lexer::Tokenizer::parse_all(&second),
                ) {
                    // Now execute the command
                    use lexer::TokenType::*;
                    use std::str::FromStr;

                    match command {
                        "mov" => {
                            // TODO :: implement it for ports and sfrs
                            let src = match op1.token {
                                HEX(hex) => Some(hex as u8),
                                IMM(_) => None,
                                ID(reg) => {
                                    // This is the direct register addressing mode .. it should be a register
                                    // try parsing it as a scratchpad register
                                    let reg = Sim8051::ScratchpadRegisters::from_str(reg.as_str())
                                        .expect("Not a scratchpad register.. error");
                                    // TODO :: Handle it for port
                                    // Return its location depending upon the currently selected register bank
                                    let pswloc =
                                        Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::PSW))
                                            as usize;
                                    let count =
                                        (0x18 & self.simulator.internal_memory.memory[pswloc]) >> 3;
                                    let start = count * 8;
                                    Some(start + reg.reg_count())
                                }
                                // For indirect addressing, retrieve the value of the register to use as src location
                                IND(reg) => {
                                    let pswloc =
                                        Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::PSW))
                                            as usize;
                                    let count =
                                        (0x18 & self.simulator.internal_memory.memory[pswloc]) >> 3;
                                    let val = count * 8 + reg.reg_count();
                                    Some(self.simulator.internal_memory.memory[val as usize])
                                }
                                _ => None,
                            };
                            // TODO :: Replace it with retrieve rvalue function
                            let dest = match op2.token {
                                HEX(hex) => {
                                    Some(self.simulator.internal_memory.memory[hex as usize])
                                }
                                IMM(hex) => Some(hex as u8), // This is the error but can't return anything here .. so changing the return type
                                ID(reg) => {
                                    // This is the direct register addressing mode .. it should be a register
                                    // try parsing it as a scratchpad register
                                    let reg = Sim8051::ScratchpadRegisters::from_str(reg.as_str())
                                        .expect("Not a scratchpad register.. error");
                                    // Return its location depending upon the currently selected register bank
                                    let pswloc =
                                        Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::PSW))
                                            as usize;
                                    let count =
                                        (0x18 & self.simulator.internal_memory.memory[pswloc]) >> 3;
                                    let start = (count * 8) as usize;
                                    Some(
                                        self.simulator.internal_memory.memory
                                            [start + reg.reg_count() as usize],
                                    )
                                }
                                // For indirect addressing, retrieve the value of the register to use as src location
                                IND(reg) => {
                                    let pswloc =
                                        Sim8051::sfr_addr(&Sim8051::SFR::Reg(Sim8051::IRegs::PSW))
                                            as usize;
                                    let count =
                                        (0x18 & self.simulator.internal_memory.memory[pswloc]) >> 3;
                                    let val = count * 8 + reg.reg_count();
                                    Some(
                                        self.simulator.internal_memory.memory[self
                                            .simulator
                                            .internal_memory
                                            .memory[val as usize]
                                            as usize],
                                    )
                                }
                                _ => None,
                            };
                            self.simulator.mov(src.unwrap(), dest.unwrap());
                            success = true;
                        },
                        // where's the single binding?
                        inst @ "add" | inst @ "addc" | inst @ "subb" => {
                            // First operand is always A .. makes one thing easy
                            // lol .. need to update flags accordingly now
                            // There are four main flags that need to be set :
                            // C, AC, OV and P :- Getting boooorring now

                            if first == "A" {
                                let operand = lexer::retrieve_rvalue(&mut self.simulator,&op2.token);
                                // Allow addition with wraparound effect
                                let addr     = Sim8051::sfr_addr(&self.simulator.accumulator);
                                let mut val  = self.simulator.internal_memory.memory[addr as usize] as i32;
                                let mut should_set_carry = false;
                                let mut factor = 1;
                                if inst == "addc" || inst == "subb"{
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

                                let temp    = val;
                                val         = val + factor *  operand.unwrap() as i32;
                                let ans     = (val & 0xff) as u8;
                                self.simulator.internal_memory.memory[addr as usize] = ans;
                                // setting these flags is plain pain
                                use std::ops::BitOr;
                                should_set_carry = should_set_carry.bitor((val as i32 & 0xff00) > 0);
                                self.simulator.set_carry_bit(should_set_carry);
                                self.simulator.set_parity_bit(ans);
                                // TODO :: Set these two stupid flags (one not stupid)

                                let should_set_parity = ((temp & 0x000F + operand.unwrap() as i32 & 0x000F) >> 4) > 0;
                                self.simulator.set_auxiliary_carry_bit(should_set_parity);

                                // lastly overflow bit
                                // overflow bit is set when there's overflow from 7th bit or 8th bit but not from both
                                let is_carry_to_msb = ((temp & 0x007F) + (operand.unwrap() as i32 & 0x007F) >> 7) > 0;
                                use std::ops::BitXor;
                                self.simulator.set_auxiliary_carry_bit(should_set_carry.bitxor(is_carry_to_msb));
                            }
                            else {
                                println!("Invalid operand to {} instruction ",inst);
                                success = false;
                            }
                        },
                        // "subb" => { // I guess subb can be merged with addc and add instructions // merged above
                        _ => success = false
                    }
                }
            }
        }
        success
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
