use std::{fs::File, io::Read};

use crate::{Sim8051::{self, InternalMemory}, lexer::{self, Tokenizer}};

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
                        self.tstmt(command,ins)
                    } else {
                        // check for single instruction command here
                        match command {
                            // since clr,  setb and cpl are quite similar, they can be merged
                            "clr"  => clr_set_cpl(self, command,ins.as_str(),InternalMemory::reset_bit_addressable),
                            "cpl"  => clr_set_cpl(self, command, ins.as_str(),InternalMemory::complement_bit_addressable),
                            "setb" => clr_set_cpl(self, command, ins.as_str(), InternalMemory::set_bit_addressable),
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
                                        true
                                    }
                                    _ => {
                                        println!("Invalid argument to rr");
                                        false
                                    }
                                }
                            },
                            "jb" => {
                                true
                            },
                            "jnb" => {
                                true
                            }
                            "jbc" => {
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
        let next_token = self.tokenizer.parse_all_as_id();
        match next_token {
            None => false,
            Some(tok) => {
                if let lexer::TokenType::ID(id) = &tok.token {
                    false
                } else {
                    true
                }
            }
        }
    }
}


fn clr_set_cpl(asm : &mut Assembler,ins : &str, operand : &str, operator : fn(&mut InternalMemory,u8,u8) ) -> bool {
    if (ins == "cpl") || (ins == "clr") {
        if operand == "A" {
            // Clear the contents of the accumulator
            let loc =
                Sim8051::sfr_addr(&asm.simulator.accumulator);
            for i in 0..8 {
                asm.simulator.internal_memory.operate_bit_addressable_memory(loc+i, operator);
            return true;
            }
        }
    }
    match operand {
        "C" => {
            // Reset the carry flag
            let psw = Sim8051::SFR::Reg(Sim8051::IRegs::PSW);
            asm.simulator.internal_memory.operate_bit_addressable_registers(Sim8051::sfr_addr(&psw), 7, operator);
            true
        }
        // It also support resetting of bit addressable memory
        rstr => {
            if let Some(hex) = lexer::Tokenizer::parse_hex(rstr) {
                asm.simulator.internal_memory.operate_bit_addressable_memory(hex as u8,operator);
                true
            }
            // TODO :: Check for bit addressability
            else {
                // Try to parse it as bit addressable registers
                if let Some(bitaddr) = Tokenizer::parse_bitaddr(rstr) {
                    match bitaddr.token {
                        lexer::TokenType::BIT_ADDR(sfr,bit) => {
                            asm.simulator.internal_memory.operate_bit_addressable_registers(Sim8051::sfr_addr(&sfr), bit, operator);
                            true
                        },
                        _   => false
                    }
                }
                else {
                    println!("Error not a bit address");
                    false
                }
            }
        }
    }
}
