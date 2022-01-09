use std::io::Read;

// Make valid tokens here and use them in the parser
// Lets work in regex for tokenizing
use crate::Sim8051;

enum TokenType
{
    ID(String),
    IMM(u8,u16),
    COMMA, LINE,
    HEX(u16),
    IND(u8),
    BIT_ADDR(Sim8051::SFR,u8),
    LABEL(String),
    INVALID,
    NULL
}

// It will only return the token for now .. More thing to be done on the parser side from here
struct Token {
    token : TokenType,
    len : usize
}

struct Tokenizer
{
    src : String,
    pos : usize,
}

impl Tokenizer
{
    pub fn parse_next(&mut self) -> Option<Token>
    {
        // Might need to implement a DFA fist .. will be doing that in class today
        let mut buf = String::with_capacity(50);
        let mut ptr = self.src.chars().skip(self.pos).peekable();
        // Tokenize the input stream now
        loop {
            match ptr.peek() {
                None    => {
                    // TODO :: handle this branch
                    return None;
                }

                Some(y)    => {
                    return match  *y {
                        '#' => {
                            // Expect hex digit here, parse that and return Hex type token
                            None
                        },
                        '@' => None,
                        ',' => None,
                        '\n' | '\r' => None, // Maybe we could ignore these white spaces / newlines for pasrsing
                        ch  => None
                    }
                }
            }
        }

        self.pos += 1;
        println!("Parsed string is : {}.",buf);
        // Lets start with a DFA
        None
    }
    pub fn read_file(&mut self, src : String)
    {
        let mut file = std::fs::File::open(src).expect("Cannot open input src file ");
        file.read_to_string(&mut self.src).expect("Failed to read input file");
    }

    pub fn parse_hex(&self) -> Option<u16> {
        let mut ptr = self.src.chars().skip(self.pos).peekable();
        let mut num = Some(0 as u16);

        loop {
            match ptr.peek() {
                None    => {
                    num = None;
                    break;
                },

                Some(y) =>
                    match *y {
                        ch @ '0'..='9' => {
                            num = Some((num.unwrap() << 4) | (ch as u16 - '0' as u16))
                        },
                        ch @ 'A'..='F' => {
                            num = Some((num.unwrap() << 4) | (ch as u16 - 'A' as u16 + 10))
                        },
                        'H'            => break,
                        _              => {
                            num = None;
                            break;
                        }
                    }
            }
            ptr.next();
        }
        return num;
    }

    // try parsing as id first
    pub fn parse_id(&self) -> Option<Token>
    {
        // First letter should be alphabetic
        let mut ptr = self.src.chars().skip(self.pos).skip_while(|x| x.is_ascii_whitespace());
        if let Some(y) = ptr.next()
        {
            if !y.is_ascii_alphabetic() {
                return None;
            }
        }
        else
        {
            return None;
        }

        let mut len = 0;

        let mut ptr = self.src.chars().skip(self.pos).enumerate().skip_while(|x| x.1.is_ascii_whitespace()).peekable();
        if let Some(y) = ptr.peek()
        {
            len = y.0;
        }

        let done = ptr.take_while(|x| x.1.is_ascii_alphanumeric());

        let buf : String = done.map(|(_,ch)| ch).collect();
        len += buf.len();
        return Some(Token{
            token : TokenType::ID(buf),
            len
        });
    }

    pub fn parse_label(&self) -> Option<Token>
    {
        let mut str = self.parse_id();
        let mut token : Option<Token> = None;

        // if let Some(y) = str
        // {
        //     // advance the iterator
        //     let ptr = self.src.chars().skip(self.pos + y.len).peekable();
        //     token = match ptr.peek() {
        //         None    => None,
        //         Some(z) => {

        //             None
        //         }
        //     }
        // }
        // None;

        // Rewrite
        token = match str {
            None      => None,
            Some(tok) => match tok.token {
                TokenType::ID(mut id) => {
                    let mut ptr = self.src.chars().skip(self.pos+tok.len).peekable();
                    match ptr.peek() {
                        None     => None,
                        Some(ch) => {
                            if ch == &';' {
                                id.push(':');
                                Some(Token{
                                    token : TokenType::LABEL(id),
                                    len : tok.len + 1
                                })
                            }
                            else {
                                None
                            }
                        }
                    }
                }
                _      => None
            }
        };
       token
    }
}
pub fn nothing()
{
    println!("Nothing here bakaa...");

}

pub fn string_handling()
{
    let mut str = String::new();
    str = String::from("Hello from The Rust Programming Language");
    unsafe {
        for ch in str.as_bytes_mut().iter_mut()
        {
            *ch = ch.to_ascii_uppercase();
        }
    }
    let z = vec![1,2,3,4];
    let mut it = z.iter().peekable();
    loop {
        let val = it.next();
        match it.peek() {
            Some(x) => println!("{} & {}",val.unwrap(),x),
            None    => break

        }
    }

    let buf = str.char_indices().skip(10).peekable();
    for (i,z) in buf
    {
        println!("-> {} {}",i,z);
    }
    println!("Total uppercase is : {}.",str);

    let mut tokenizer = Tokenizer { src : String::from("The Rust Programming Language"),
                                pos : 0
    };
    tokenizer.parse_next();
    tokenizer.parse_next();
    tokenizer.parse_next();
    tokenizer.parse_next();

    let z = (4,"Four");
    println!("{} -> {}",z.0,z.1);

    // Parsing test
    let hextokenizer = Tokenizer { src : String::from("ABZH"), pos : 0};
    let z = hextokenizer.parse_hex();
    match z
    {
        Some(x) => println!("Found hex value : {}.",x),
        None    => println!("Found no hex value -_-")
    }

    let idtokenizer = Tokenizer { src : String::from("  Something wild"), pos : 0 };
    idtokenizer.parse_id();

    let newvec = vec!(1,2,3,4,5,6);
    let mut pvec = newvec.iter();
    let res = pvec.take_while(|x| **x < 3);
    for i in res {
        println!("i -> {}",i);
    }

    // string testing
    let mut test = String::from("   The RUsty");
    let mut ptr = test.chars().enumerate().skip_while(|x| x.1.is_ascii_whitespace()).take_while(|x| x.1.is_ascii_uppercase());
    // match ptr.next()
    // {
    //     Some(y) => println!("0 -> {}, 1 -> {}",y.0,y.1),
    //     None    => println!("Nothing found here .. sad")
    // }
    for i in ptr
    {
        println!("From loop : {} {}",i.0,i.1);
    }

}
