use std::{fs::File, io::Read};

use crate::lexer;

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
    pub tokenizer: lexer::Tokenizer,
}

impl Default for Assembler {
    fn default() -> Assembler {
        Assembler { tokenizer : lexer::Tokenizer::default() }
    }
}


impl Assembler {
    // This is the beginning of the parser
    pub fn read_src(&mut self, srcfile : String) -> bool {
        let mut src = File::open(srcfile);
        match src {
            Ok(mut file) => {
                file.read_to_string(&mut self.tokenizer.src).expect("Failed to read the open file");
                true
            },
            Err(_)    => false
        }
    }

    pub fn start(&mut self) {
        let mut ptr = self.tokenizer.src.chars();
        if let Some(ch) = ptr.next() {
            if ch != '^' {
                println!("Unexpected character at the beginning of the file");
                return;
            }
        }
        else
        {
            return;
        }
        self.tokenizer.pos += 1;
        self.prog();
    }
    // This function is the main core of the parser
    // I guess LL(1) grammar should work fine
    fn prog(&mut self) {
        // It can be either stmt or label with stmt or empty
        let mut ptr = self.tokenizer.src.chars().skip(self.tokenizer.pos).skip_while(|x| x.is_ascii_whitespace());
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
            None     => println!("Unexpected token found here"),

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

                        }
                        else {
                            println!("Invalid token here : ");
                        }
                    }
                    else {
                        // It has to be id
                        self.stmt(&tok);
                        self.prog();
                    }
                }
            }
        }
    }

    fn stmt(&mut self, prev_token : &lexer::Token) {
        // This token better be
        // This func now should decide to return or call the second statement
        // This resulting token better be a asm instruction
        // Lets differ match, lets just try parsing
        println!("Expanded along : stmt -> stmt prog");
        self.tokenizer.pos += prev_token.len;
        // See if the next token is comma
        if self.tokenizer.consume_comma() {
            // propage the current command to the next function
            let lexer::TokenType::ID(ins) = prev_token.token;
            self.sstmt(ins.as_str());
        } else {
            // else its the single instruction command .. execute it right here
            if let lexer::TokenType::ID(ins) = &prev_token.token {
                match ins.as_str() {
                    "mov" => println!("Found mov command"),
                    "add" => println!("Found add command"),
                    "stc" => println!("Found stc command"),
                    com   => println!("Found unknown command : {}",com)
                }
            }
        }
        self.prog();

    }

    fn sstmt(&mut self, command : &str) {
        // This function carries the previous command and is ready to decode the next operand
    }

    fn tstmt(&mut self) {
    }
}
