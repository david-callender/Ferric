use std::{collections::HashMap, iter::Peekable, vec};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let,          // let
    If,           // if
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
    Tilde,        // ~
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
            Token::Tilde => write!(f, "~"),
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
            Token::NumLit(n) => write!(f, r#""{n}""#),
            Token::Ident(n) => write!(f, "ident[{n}]"),
        }
    }
}

fn parse_number(digits: Vec<u8>) -> f64 {
    let mut num = 0.0;
    let mut i: i32 = -1;
    let mut after_dec = false;
    let mut frac_appears = false;

    assert!(digits[0] != b'.', "Missing leading zero"); //Curently .1 will panic due to '.' being an invalid byte, but this will be useful later

    for b in digits {
        if b == b'.' {
            assert!(!after_dec, "Multiple decimal points in number");
            after_dec = true;
            continue;
        }
        if b.is_ascii_digit() {
            let n = f64::from(b - b'0');
            if after_dec {
                num += n * 10f64.powi(i);
                i -= 1;
                frac_appears = true;
            } else {
                num *= 10.0;
                num += n;
            }
        }
    }
    assert!(
        (!after_dec || frac_appears),
        "No numbers detected after decimal"
    );
    num
}

pub struct Lexer<I: Iterator<Item = u8>> {
    stream: Peekable<I>,
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    pub fn new(stream: I) -> Self {
        Self {
            stream: stream.peekable(),
        }
    }

    fn lex_multi_byte(&mut self, first: u8) -> Token {
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

    fn lex_number_lit(&mut self, first: u8) -> Token {
        let mut num = Vec::new();
        num.push(first);
        while let Some(a) = self.stream.peek() {
            if a.is_ascii_digit() || *a == b'.' {
                num.push(*a);
                self.stream.next();
            } else {
                break;
            }
        }
        Token::NumLit(parse_number(num))
    }

    fn lex_ident(&mut self, first: u8) -> Token {
        let keywords = HashMap::from([
            ("let", Token::Let),
            ("let", Token::Let),
            ("if", Token::If),
            ("otherwise", Token::Otherwise),
            ("while", Token::While),
            ("fn", Token::Fn),
            ("and", Token::LAnd),
            ("or", Token::LOr),
        ]);
        let mut ident_bytes = vec![first];
        while let Some(b) = self.stream.peek()
            && (b.is_ascii_alphanumeric() || *b == b'_')
        {
            ident_bytes.push(*b);
            self.stream.next();
        }
        let ident = String::from_utf8(ident_bytes).expect("Identifier wasn't valid utf8");

        if let Some(keyword) = keywords.get(ident.as_str()) {
            keyword.clone()
        } else {
            Token::Ident(ident)
        }
    }

    fn lex_string_lit(&mut self) -> Token {
        let mut st = Vec::new();
        loop {
            let s = self.stream.next().expect("unterminated string literal");
            if s == b'"' {
                let st = String::from_utf8(st).expect("invalid UTF-8 in string literal");
                return Token::StringLit(st);
            }
            if s == b'\\' {
                let esc = self
                    .stream
                    .next()
                    .expect("expected escape sequence, got none");
                match esc {
                    b'n' => st.push(b'\n'),
                    b't' => st.push(b'\t'),
                    b'r' => st.push(b'\r'),
                    b'"' => st.push(b'"'),
                    b'\\' => st.push(b'\\'),
                    _ => panic!("Invalid escape sequence \\{}", s as char),
                }
                continue;
            }
            st.push(s);
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

            let tok = match c {
                b'(' => Token::OpenParen,
                b')' => Token::CloseParen,
                b'{' => Token::OpenBracket,
                b'}' => Token::CloseBracket,
                b';' => Token::Semi,
                b',' => Token::Comma,
                b'+' => Token::Plus,
                b'-' => Token::Minus,
                b'*' => Token::Star,
                b'/' => Token::Slash,
                b'~' => Token::Tilde,

                b'=' | b'!' | b'<' | b'>' => self.lex_multi_byte(c),

                x if x.is_ascii_digit() => self.lex_number_lit(x),
                x if x.is_ascii_alphabetic() || x == b'_' => self.lex_ident(x),
                b'"' => self.lex_string_lit(),

                b => panic!("Invalid byte {}", b as char),
            };

            return Some(tok);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token as T;

    fn collect_tokens(src: &str) -> Vec<Token> {
        Lexer::new(src.bytes()).collect()
    }

    #[test]
    fn one_byte() {
        assert_eq!(
            collect_tokens("(){};,+-*/"),
            vec![
                T::OpenParen,
                T::CloseParen,
                T::OpenBracket,
                T::CloseBracket,
                T::Semi,
                T::Comma,
                T::Plus,
                T::Minus,
                T::Star,
                T::Slash
            ]
        );
    }

    #[test]
    fn two_byte() {
        assert_eq!(
            collect_tokens("=== = == = <<=< >>=>  !=!"),
            vec![
                T::EqEq,
                T::Eq,
                T::Eq,
                T::EqEq,
                T::Eq,
                T::Less,
                T::LessEq,
                T::Less,
                T::Greater,
                T::GreaterEq,
                T::Greater,
                T::BangEq,
                T::Bang,
            ]
        );
    }

    #[test]
    fn number_lit() {
        assert_eq!(collect_tokens("1"), vec![T::NumLit(1.0)]);
        assert_eq!(collect_tokens("1.0"), vec![T::NumLit(1.0)]);
        assert_eq!(collect_tokens("001.000"), vec![T::NumLit(1.0)]);
        assert_eq!(collect_tokens("0.1"), vec![T::NumLit(0.1)]);
        assert_eq!(
            collect_tokens("1234567890"),
            vec![T::NumLit(1_234_567_890.0)]
        );
    }

    #[test]
    fn ident() {
        assert_eq!(collect_tokens("hello"), vec![T::Ident("hello".to_string())]);
        assert_eq!(collect_tokens("and"), vec![T::LAnd]);
    }

    #[test]
    fn string_lit() {
        assert_eq!(
            collect_tokens("\"string lit\""),
            vec![T::StringLit("string lit".to_string())]
        );
        assert_eq!(
            collect_tokens("\"string\\n\\tlit\""),
            vec![T::StringLit("string\n\tlit".to_string())]
        );
    }
}
