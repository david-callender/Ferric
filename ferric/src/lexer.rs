use std::{collections::HashMap, iter::Peekable, string::FromUtf8Error, vec};

use thiserror::Error;

use crate::loc::{Loc, ProgramSrc, Span};

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("{}", .1.format(.0, "number literals cannot start with '.'"))]
    NumLitLeadingDecimal(ProgramSrc, Span),

    #[error("{}", .1.format(.0, "number literals cannot end with '.'"))]
    NumLitTrailingDecimal(ProgramSrc, Span),

    #[error("{}", .1.format(.0, "number literals cannot have multiple decimal separators"))]
    NumLitMultipleDecimals(ProgramSrc, Span),

    #[error("{}", .1.format(.0, &format!("this byte ({} or {}) was not expected by the lexer", *.2 as char, .2)))]
    InvalidByte(ProgramSrc, Loc, u8),

    #[error("{}", .1.format(.0, &format!("this string literal was not valid utf-8: {}", .2)))]
    StrLitInvalidUtf8(ProgramSrc, Span, FromUtf8Error),

    #[error("{}", .1.format(.0, &format!("this identifier was not valid utf-8: {}", .2)))]
    IdentInvalidUtf8(ProgramSrc, Span, FromUtf8Error),

    #[error("{}", .1.format(.0, &format!("'\\{}' is not a valid escape sequence", *.2 as char)))]
    InvalidEscapeSequence(ProgramSrc, Span, u8),

    #[error("{}", .1.format(.0, "this string literal was not terminated"))]
    UnterminatedStrLit(ProgramSrc, Span),
}

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
    Percent,      // %
    Greater,      // >
    GreaterEq,    // >=
    Less,         // <
    LessEq,       // <=
    LAnd,         // and
    LOr,          // or
    Bang,         // !
    True,         //true
    False,        //false
    StringLit(String),
    NumLit(f64),
    Ident(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lexeme {
    pub t: Token,
    pub span: Span,
}

impl Lexeme {
    pub fn new(t: Token, span: Span) -> Self {
        Self { t, span }
    }
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
            Token::Percent => write!(f, "%"),
            Token::Greater => write!(f, ">"),
            Token::GreaterEq => write!(f, ">="),
            Token::Less => write!(f, "<"),
            Token::LessEq => write!(f, "<="),
            Token::LAnd => write!(f, "and"),
            Token::LOr => write!(f, "or"),
            Token::Bang => write!(f, "!"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::StringLit(n) => write!(f, "{n}"),
            Token::NumLit(n) => write!(f, r#""{n}""#),
            Token::Ident(n) => write!(f, "ident[{n}]"),
        }
    }
}


pub struct Lexer<I: Iterator<Item = u8>> {
    stream: Peekable<I>,
    keywords: HashMap<&'static str, Token>,
    src: ProgramSrc,
    line: usize,
    col: usize,
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    // TODO: unify into one param
    pub fn new(stream: I, src: ProgramSrc) -> Self {
        let keywords = HashMap::from([
            ("let", Token::Let),
            ("let", Token::Let),
            ("if", Token::If),
            ("otherwise", Token::Otherwise),
            ("while", Token::While),
            ("fn", Token::Fn),
            ("and", Token::LAnd),
            ("or", Token::LOr),
            ("true", Token::True),
            ("false", Token::False),
        ]);
        Self {
            stream: stream.peekable(),
            keywords,
            src,
            line: 1,
            col: 1,
        }
    }

    fn loc(&self) -> Loc {
        Loc::new(self.line, self.col)
    }

    fn next(&mut self) -> Option<(u8, Loc)> {
        let n = self.stream.next()?;
        let this = self.loc();
        if n == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some((n, this))
    }
    
    fn lex_byte(&mut self, c: u8, loc: Loc) -> Result<Lexeme, LexerError> {
        let tok = match c {
            b'(' => Lexeme::new(Token::OpenParen, loc.into()),
            b')' => Lexeme::new(Token::CloseParen, loc.into()),
            b'{' => Lexeme::new(Token::OpenBracket, loc.into()),
            b'}' => Lexeme::new(Token::CloseBracket, loc.into()),
            b';' => Lexeme::new(Token::Semi, loc.into()),
            b',' => Lexeme::new(Token::Comma, loc.into()),
            b'+' => Lexeme::new(Token::Plus, loc.into()),
            b'-' => Lexeme::new(Token::Minus, loc.into()),
            b'*' => Lexeme::new(Token::Star, loc.into()),
            b'/' => Lexeme::new(Token::Slash, loc.into()),
            b'%' => Lexeme::new(Token::Percent, loc.into()),
            b'~' => Lexeme::new(Token::Tilde, loc.into()),

            b'=' | b'!' | b'<' | b'>' => self.lex_multi_byte(c, loc),

            x if x.is_ascii_digit() => self.lex_number_lit(x, loc)?,
            x if x.is_ascii_alphabetic() || x == b'_' => self.lex_ident(x, loc)?,
            b'"' => self.lex_string_lit(loc)?,

            b => return Err(LexerError::InvalidByte(self.src.clone(), loc, b)),
        };
        Ok(tok)
    }

    fn lex_multi_byte(&mut self, first: u8, loc: Loc) -> Lexeme {
        let second = self.stream.peek();
        match (first, second) {
            (b'=', Some(b'=')) => {
                let (_, snd) = self.next().unwrap();
                Lexeme::new(Token::EqEq, loc + snd)
            }
            (b'<', Some(b'=')) => {
                let (_, snd) = self.next().unwrap();
                Lexeme::new(Token::LessEq, loc + snd)
            }
            (b'>', Some(b'=')) => {
                let (_, snd) = self.next().unwrap();
                Lexeme::new(Token::GreaterEq, loc + snd)
            }
            (b'!', Some(b'=')) => {
                let (_, snd) = self.next().unwrap();
                Lexeme::new(Token::BangEq, loc + snd)
            }
            (b'=', _) => Lexeme::new(Token::Eq, loc.into()),
            (b'<', _) => Lexeme::new(Token::Less, loc.into()),
            (b'>', _) => Lexeme::new(Token::Greater, loc.into()),
            (b'!', _) => Lexeme::new(Token::Bang, loc.into()),
            _ => panic!("Unreachable: invalid start byte in multi-byte call {}", first as char),
        }
    }

    fn lex_number_lit(&mut self, first: u8, first_loc: Loc) -> Result<Lexeme, LexerError> {
        let mut num = Vec::new();
        num.push(first);
        let mut span: Span = first_loc.into();
        while let Some(a) = self.stream.peek() {
            if a.is_ascii_digit() || *a == b'.' {
                num.push(*a);
                let (_, loc) = self.next().unwrap();
                span = span + loc;
            } else {
                break;
            }
        }
        Ok(Lexeme::new(Token::NumLit(self.parse_number(num, span.clone())?), span))
    }
    
    fn parse_number(&mut self, digits: Vec<u8>, span: Span) -> Result<f64, LexerError> {
        let mut num = 0.0;
        let mut i: i32 = -1;
        let mut after_dec = false;
        let mut frac_appears = false;
        
        if digits[0] == b'.' {
            return Err(LexerError::NumLitLeadingDecimal(self.src.clone(), span));
        }
    
        for b in digits {
            if b == b'.' {
                if after_dec {
                    return Err(LexerError::NumLitMultipleDecimals(self.src.clone(), span));
                }
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
        if after_dec && !frac_appears {
            return Err(LexerError::NumLitTrailingDecimal(self.src.clone(), span));
        }
        Ok(num)
    }

    fn lex_ident(&mut self, first: u8, first_loc: Loc) -> Result<Lexeme, LexerError> {
        let mut ident_bytes = vec![first];
        let mut span: Span = first_loc.into();
        while let Some(b) = self.stream.peek()
            && (b.is_ascii_alphanumeric() || *b == b'_')
        {
            ident_bytes.push(*b);
            let (_, loc) = self.next().unwrap();
            span = span + loc;
        }
        let ident = String::from_utf8(ident_bytes).map_err(|err| LexerError::IdentInvalidUtf8(self.src.clone(), span.clone(), err))?;

        let tok = if let Some(keyword) = self.keywords.get(ident.as_str()) {
            keyword.clone()
        } else {
            Token::Ident(ident)
        };

        Ok(Lexeme::new(tok, span))
    }

    fn lex_string_lit(&mut self, first: Loc) -> Result<Lexeme, LexerError> {
        let mut st = Vec::new();
        let mut span: Span = first.into();
        loop {
            let (s, loc) = self.next().ok_or(LexerError::UnterminatedStrLit(
                self.src.clone(),
                span.clone(),
            ))?;
            span = span + loc;
            if s == b'"' {
                let st = String::from_utf8(st).map_err(|err| {
                    LexerError::StrLitInvalidUtf8(self.src.clone(), span.clone(), err)
                })?;
                return Ok(Lexeme::new(Token::StringLit(st), span));
            }
            if s == b'\\' {
                // should be made into its own error
                let (esc, loc) = self.next().ok_or(LexerError::InvalidEscapeSequence(self.src.clone(), span.clone(), b' '))?;
                span = span + loc;
                match esc {
                    b'n' => st.push(b'\n'),
                    b't' => st.push(b'\t'),
                    b'r' => st.push(b'\r'),
                    b'"' => st.push(b'"'),
                    b'\\' => st.push(b'\\'),
                    b => return Err(LexerError::InvalidEscapeSequence(self.src.clone(), span, b)),
                }
                continue;
            }
            st.push(s);
        }
    }
}

impl<I: Iterator<Item = u8>> Iterator for Lexer<I> {
    type Item = Result<Lexeme, LexerError>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (c, loc) = self.next()?;
            if c.is_ascii_whitespace() {
                continue;
            }
            
            let tok = self.lex_byte(c, loc);

            return Some(tok);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token as T;

    fn collect_tokens(src: &str) -> Vec<Token> {
        let src = ProgramSrc::new(src.to_string());
        Lexer::new(src.clone().stream(), src)
            .map(|lx| lx.unwrap().t)
            .collect()
    }

    #[test]
    fn one_byte() {
        assert_eq!(
            collect_tokens("(){};,+-*/%"),
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
                T::Slash,
                T::Percent
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
