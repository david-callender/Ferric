use std::{collections::HashMap, iter::Peekable};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let,          // let
    If,           // if
    Elseif,       // elseif
    Otherwise,    // otherwise
    While,        // while
    Fn,           // fn
    OpenParen,    // (
    CloseParen,   // )
    OpenBracket,  // {
    CloseBracket, // }
    Semi,         // ;
    Comma,        // ,
    Eq,           // =
    EqEq,         // ==
    BangEq,       // !=
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Greater,      // >
    GreaterEq,    // >=
    Less,         // <
    LessEq,       // <=
    LAnd,         // and
    LOr,          // or
    Bang,         // !
    StringLit(String),
    NumLit(f64),
    Ident(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Let => write!(f, "let"),
            Token::If => write!(f, "if"),
            Token::Elseif => write!(f, "elseif"),
            Token::Otherwise => write!(f, "otherwise"),
            Token::While => write!(f, "while"),
            Token::Fn => write!(f, "fn"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenBracket => write!(f, "{{"),
            Token::CloseBracket => write!(f, "}}"),
            Token::Semi => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::BangEq => write!(f, "!="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Greater => write!(f, ">"),
            Token::GreaterEq => write!(f, ">="),
            Token::Less => write!(f, "<"),
            Token::LessEq => write!(f, "<="),
            Token::LAnd => write!(f, "and"),
            Token::LOr => write!(f, "or"),
            Token::Bang => write!(f, "!"),
            Token::StringLit(n) => write!(f, "{n}"),
            Token::NumLit(n) => write!(f, "\"{n}\""),
            Token::Ident(n) => write!(f, "ident[{n}]"),
        }
    }
}

pub struct Lexer<I: Iterator<Item = u8>> {
    stream: Peekable<I>,
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    fn matching(&mut self, first: u8) -> Token {
        let second = self.stream.peek();
        match (first, second) {
            (b'=', Some(b'=')) => {
                self.stream.next();
                Token::EqEq
            }
            (b'<', Some(b'=')) => {
                self.stream.next();
                Token::LessEq
            }
            (b'>', Some(b'=')) => {
                self.stream.next();
                Token::GreaterEq
            }
            (b'!', Some(b'=')) => {
                self.stream.next();
                Token::BangEq
            }
            (b'=', _) => Token::Eq,
            (b'<', _) => Token::Less,
            (b'>', _) => Token::Greater,
            (b'!', _) => Token::Bang,
            _ => panic!("Invalid byte {}", first as char),
        }
    }
}

impl<I: Iterator<Item = u8>> Iterator for Lexer<I> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.stream.next()?;
            if c.is_ascii_whitespace() {
                continue;
            }
            match c {
                b'(' => return Some(Token::OpenParen),
                b')' => return Some(Token::CloseParen),
                b'{' => return Some(Token::OpenBracket),
                b'}' => return Some(Token::CloseBracket),
                b';' => return Some(Token::Semi),
                b',' => return Some(Token::Comma),
                b'+' => return Some(Token::Plus),
                b'-' => return Some(Token::Minus),
                b'*' => return Some(Token::Star),
                b'/' => return Some(Token::Slash),
                b'=' | b'!' | b'<' | b'>' => return Some(self.matching(c)),
                x if x.is_ascii_digit() => {
                    let mut num = String::new();
                    num.push(x as char);
                    while let Some(a) = self.stream.peek() {
                        if a.is_ascii_digit() || *a == b'.' {
                            num.push(*a as char);
                            self.stream.next();
                        } else {
                            break;
                        }
                    }
                    return Some(Token::NumLit(num.parse::<f64>().unwrap()));
                }
                x if x.is_ascii_alphabetic() || x == b'_' => {
                    let keywords = HashMap::from([
                        ("let", Token::Let),
                        ("let", Token::Let),
                        ("if", Token::If),
                        ("elseif", Token::Elseif),
                        ("otherwise", Token::Otherwise),
                        ("while", Token::While),
                        ("fn", Token::Fn),
                    ]);
                    let mut ident_bytes = vec![x];
                    while let Some(b) = self.stream.peek()
                        && (b.is_ascii_alphanumeric() || *b == b'_')
                    {
                        ident_bytes.push(*b);
                        self.stream.next();
                    }
                    let ident =
                        String::from_utf8(ident_bytes).expect("Identifier wasn't valid utf8");

                    return if let Some(keyword) = keywords.get(ident.as_str()) {
                        Some(keyword.clone())
                    } else {
                        Some(Token::Ident(ident))
                    };
                }
                b'"' => {
                    let mut st = String::new();
                    loop {
                    let s = self.stream.next().expect("expected closing quote, got none");
                        if s == b'"' {
                            return Some(Token::StringLit(st));
                        }
                        if s == b'\\'{
                            let esc = self.stream.next().expect("expected escape sequence, got none");
                            match esc {
                                b'n' => st.push('\n'),
                                b't' => st.push('\t'),
                                b'r' => st.push('\r'),
                                b'"' => st.push('"'),
                                b'\\' => st.push('\\'),
                                _ => panic!("Invalid escape sequence"),
                            }
                            continue;
                        }
                        st.push(s as char);
                    }
                }
                b => panic!("Invalid byte {}", b as char),
            }
        }
    }
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    pub fn new(stream: I) -> Self {
        Self {
            stream: stream.peekable(),
        }
    }
}
