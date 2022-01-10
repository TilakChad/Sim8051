use std::io::Read;

// Make valid tokens here and use them in the parser
// Lets work in regex for tokenizing
use crate::Sim8051;

#[derive(Debug)]
pub enum TokenType {
    ID(String),
    IMM(u8, u16),
    COMMA,
    LINE,
    HEX(u16),
    IND(u8),
    BIT_ADDR(Sim8051::SFR, u8),
    LABEL(String),
    INVALID,
    NULL,
}

// It will only return the token for now .. More thing to be done on the parser side from here
#[derive(Debug)]
pub struct Token {
    token: TokenType,
    len: usize,
}

pub struct Tokenizer {
    pub src: String,
    pub pos: usize,
}

impl Tokenizer {
    pub fn parse_next(&mut self) -> Option<Token> {
        // Might need to implement a DFA fist .. will be doing that in class today
        let mut buf = String::with_capacity(50);
        // TODO :: If newline, return newline token ..
        // Now we going for parser and parsing
        let mut ptr = self
            .src
            .chars()
            .skip(self.pos)
            .skip_while(|x| x.is_ascii_whitespace())
            .peekable();
        let mut count = self
            .src
            .chars()
            .skip(self.pos)
            .take_while(|x| x.is_ascii_whitespace())
            .count();
        self.pos += count;
        // Tokenize the input stream now
        let mut token = None;
        println!("Tokenizer called here with skip : {}", self.pos);

        // Consume white space character here
        token = match ptr.peek() {
            None => None,
            Some(y) => {
                match *y {
                    '#' => {
                        self.pos += 1;
                        self.parse_hex()
                    }
                    '@' => {
                        self.pos += 1;
                        self.parse_id()
                    }
                    ',' => Some(Token {
                        token: TokenType::COMMA,
                        len: 1,
                    }),
                    '\n' | '\r' => Some(Token {
                        token: TokenType::LINE,
                        len: 1,
                    }), // Maybe we could ignore these white spaces / newlines for pasrsing
                    _ => self.parse_id(),
                }
            }
        };

        if let Some(ref y) = token {
            self.pos += y.len;
        }

        // Lets start with a DFA
        token
    }
    pub fn read_file(&mut self, src: String) {
        let mut file = std::fs::File::open(src).expect("Cannot open input src file ");
        file.read_to_string(&mut self.src)
            .expect("Failed to read input file");
    }

    pub fn parse_hex(&self) -> Option<Token> {
        let mut ptr = self.src.chars().skip(self.pos).peekable();
        let mut num: u16 = 0;

        let mut token = None;
        let mut count = 0;
        loop {
            match ptr.peek() {
                None => {
                    break;
                }

                Some(y) => match *y {
                    ch @ '0'..='9' => num = (num << 4) | (ch as u16 - '0' as u16),
                    ch @ 'A'..='F' => num = (num << 4) | (ch as u16 - 'A' as u16 + 10),
                    'H' => {
                        token = Some(Token {
                            token: TokenType::HEX(num),
                            len: count + 1,
                        });
                        break;
                    }
                    _ => {
                        token = None;
                        break;
                    }
                },
            }
            count += 1;
            ptr.next();
        }
        token
    }

    // try parsing as id first
    pub fn parse_id(&self) -> Option<Token> {
        // First letter should be alphabetic
        let mut ptr = self
            .src
            .chars()
            .skip(self.pos)
            .skip_while(|x| x.is_ascii_whitespace());
        if let Some(y) = ptr.next() {
            if !y.is_ascii_alphabetic() {
                return None;
            }
        } else {
            return None;
        }

        let mut len = 0;

        let mut ptr = self
            .src
            .chars()
            .skip(self.pos)
            .enumerate()
            .skip_while(|x| x.1.is_ascii_whitespace())
            .peekable();
        if let Some(y) = ptr.peek() {
            len = y.0;
        }

        let done = ptr.take_while(|x| x.1.is_ascii_alphanumeric());

        let buf: String = done.map(|(_, ch)| ch).collect();
        len += buf.len();
        return Some(Token {
            token: TokenType::ID(buf),
            len,
        });
    }

    pub fn parse_label(&self) -> Option<Token> {
        let mut str = self.parse_id();
        let mut token: Option<Token> = None;
        // Rewrite
        token = match str {
            None => None,
            Some(tok) => match tok.token {
                TokenType::ID(mut id) => {
                    let mut ptr = self.src.chars().skip(self.pos + tok.len).peekable();
                    match ptr.peek() {
                        None => None,
                        Some(ch) => {
                            if *ch == ':' {
                                id.push(':');
                                Some(Token {
                                    token: TokenType::LABEL(id),
                                    len: tok.len + 1,
                                })
                            } else {
                                None
                            }
                        }
                    }
                }
                _ => None,
            },
        };
        token
    }

    pub fn parse_bitaddr(&self) -> Option<Token> {
        // Its syntax is something followed by dot and then followed by a single number .. Nothing more
        use std::str::FromStr;
        let addressable = self.parse_id();
        let mut token = None;
        token = match addressable {
            None => None,
            Some(tok) => match tok.token {
                TokenType::ID(str) => {
                    let mut ptr = self.src.chars().skip(self.pos + tok.len).peekable();
                    match ptr.peek() {
                        None => None,
                        Some(z) => {
                            if *z == '.' {
                                ptr.next();
                                // Continue parsing toward a number
                                if let Some(ch) = ptr.next() {
                                    println!("Values are {} and {}.", ch as u8, '0' as u8);
                                    let var = ch as u8 - '0' as u8;
                                    if var < 10 {
                                        Some(Token {
                                            token: TokenType::BIT_ADDR(
                                                Sim8051::SFR::from_str(&str)
                                                    .expect("Not a bitaddressable variable"),
                                                var,
                                            ),
                                            len: tok.len + 2,
                                        })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    }
                }
                _ => None,
            },
        };
        token
    }

    pub fn consume_comma(&mut self) -> bool {
        let ptr = self.src.chars().skip(self.pos).next();
        match ptr {
            None => false, // throw some kind of error here
            Some(ch) => {
                if ch == ',' {
                    self.pos += 1;
                    return true;
                }
                false
            }
        }
    }

    pub fn consume_newlines(&mut self) -> bool {
        let ptr = self.src.chars().skip(self.pos);
        let count = ptr.take_while(|x| x.is_ascii_whitespace()).count();
        self.pos = self.pos + count;
        match count {
            0 => false,
            _ => true,
        }
    }
}

pub fn nothing() {
    println!("Nothing here bakaa...");
}

pub fn string_handling() {
    let mut str = String::new();
    str = String::from("Hello from The Rust Programming Language");
    unsafe {
        for ch in str.as_bytes_mut().iter_mut() {
            *ch = ch.to_ascii_uppercase();
        }
    }
    let z = vec![1, 2, 3, 4];
    let mut it = z.iter().peekable();
    loop {
        let val = it.next();
        match it.peek() {
            Some(x) => println!("{} & {}", val.unwrap(), x),
            None => break,
        }
    }

    let buf = str.char_indices().skip(10).peekable();
    for (i, z) in buf {
        println!("-> {} {}", i, z);
    }
    println!("Total uppercase is : {}.", str);

    let mut tokenizer = Tokenizer {
        src: String::from("The Rust Programming Language"),
        pos: 0,
    };
    tokenizer.parse_next();
    tokenizer.parse_next();
    tokenizer.parse_next();
    tokenizer.parse_next();

    let z = (4, "Four");
    println!("{} -> {}", z.0, z.1);

    // Parsing test
    let hextokenizer = Tokenizer {
        src: String::from("ABZH"),
        pos: 0,
    };
    // let z = hextokenizer.parse_hex();
    // match z {
    //     Some(x) => println!("Found hex value : {}.", x),
    //     None => println!("Found no hex value -_-"),
    // }

    let idtokenizer = Tokenizer {
        src: String::from("  Something wild"),
        pos: 0,
    };
    idtokenizer.parse_id();

    let newvec = vec![1, 2, 3, 4, 5, 6];
    let mut pvec = newvec.iter();
    let res = pvec.take_while(|x| **x < 3);
    for i in res {
        println!("i -> {}", i);
    }

    // string testing
    let mut test = String::from("   The RUsty");
    let mut ptr = test
        .chars()
        .enumerate()
        .skip_while(|x| x.1.is_ascii_whitespace())
        .take_while(|x| x.1.is_ascii_uppercase());
    // match ptr.next()
    // {
    //     Some(y) => println!("0 -> {}, 1 -> {}",y.0,y.1),
    //     None    => println!("Nothing found here .. sad")
    // }
    for i in ptr {
        println!("From loop : {} {}", i.0, i.1);
    }

    // Parse tests
    let mut labeltest = Tokenizer {
        src: String::from("P2.1 a,b"),
        pos: 0,
    };

    match labeltest.parse_bitaddr() {
        None => println!("Failed to parse given token "),
        Some(z) => println!("Found some token :-> {:?}.", z),
    }

    // let f = TokenType::BIT_ADDR(Sim8051::SFR::Port(Sim8051::Ports::P0),3);
    // println!("In Debug format {:?}.",f);
    let mut test = Tokenizer {
        src: String::from("mov @R0, #22H \n mov R1, #34H"),
        pos: 0,
    };

    loop {
        match test.parse_next() {
            None => break,
            Some(token) => println!("Found token : {:?}", token),
        }
    }
}
