use std::iter::Peekable;

use crate::{interpreter::RuntimeVal, lexer::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(RuntimeVal),
}

pub struct Parser<I: Iterator<Item = Token>> {
    stream: Peekable<I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(stream: I) -> Self {
        Self {
            stream: stream.peekable(),
        }
    }

    fn consume(&mut self, expected: Token, message: &str) -> Token {
        let token = self.stream.next().expect("expected token, got none");
        assert_eq!(token, expected, "{message}");
        drop(expected);
        token
    }

    pub fn parse(&mut self) -> Expr {
        self.parse_basic()
    }

    fn parse_basic(&mut self) -> Expr {
        let token = self.stream.next().expect("expected basic token, got none");
        match token {
            Token::OpenParen => self.parse_paren(),
            Token::StringLit(string) => Expr::Literal(RuntimeVal::String(string)),
            Token::NumLit(number) => Expr::Literal(RuntimeVal::Number(number)),
            _ => panic!("expected basic token, got non-basic token"),
        }
    }

    // parse_paren assumes that the initial OpenParen token has already
    // been consumed.
    fn parse_paren(&mut self) -> Expr {
        let inner_expr = self.parse();
        self.consume(Token::CloseParen, "unclosed paren block");
        inner_expr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_num_literal() {
        let mut parser = Parser::new([Token::NumLit(4.0)].into_iter());
        assert_eq!(parser.parse(), Expr::Literal(RuntimeVal::Number(4.0)));
    }

    #[test]
    pub fn test_string_literal() {
        let mut parser = Parser::new([Token::StringLit("dingus".to_string())].into_iter());
        assert_eq!(
            parser.parse(),
            Expr::Literal(RuntimeVal::String("dingus".to_string()))
        );
    }

    #[test]
    pub fn test_parentheses() {
        let mut parser =
            Parser::new([Token::OpenParen, Token::NumLit(4.0), Token::CloseParen].into_iter());
        assert_eq!(parser.parse(), Expr::Literal(RuntimeVal::Number(4.0)));
    }
}
