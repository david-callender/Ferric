use std::iter::Peekable;

use crate::{interpreter::RuntimeVal, lexer::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(RuntimeVal),
    Binary {
        left: Box<Expr>,
        operation: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        operation: UnaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    BitNot,
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

    fn matches(&mut self, expected: Token) -> bool {
        if self.stream.peek() == Some(&expected) {
            let _ = self.stream.next().unwrap();
            drop(expected);
            true
        } else {
            false
        }
    }

    fn consume(&mut self, expected: Token, message: &str) -> Token {
        let token = self.stream.next().expect("expected token, got none");
        assert_eq!(token, expected, "{message}");
        drop(expected);
        token
    }

    pub fn parse(&mut self) -> Expr {
        self.parse_add()
    }

    fn parse_add(&mut self) -> Expr {
        let left = self.parse_multiplication();

        if self.matches(Token::Plus) {
            let right = self.parse_multiplication();
            return Expr::Operation {
                left: Box::new(left),
                operation: BinaryOp::Add,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_multiplication(&mut self) -> Expr {
        let left = self.parse_basic();
        if self.matches(Token::Star) {
            let right = self.parse_basic();
            return Expr::Operation {
                left: Box::new(left),
                operation: Operator::Multiply,
                right: Box::new(right),
            };
        }
        left
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

    #[test]
    pub fn test_add() {
        let mut parser =
            Parser::new([Token::NumLit(4.0), Token::Plus, Token::NumLit(5.0)].into_iter());
        let target = Expr::Binary{
            left: Box::new(Expr::Literal(RuntimeVal::Number(4.0))),
            operation: BinaryOp::Add,
            right: Box::new(Expr::Literal(RuntimeVal::Number(5.0))),
        };
        assert_eq!(parser.parse(), target);
    }

    #[test]
    pub fn test_multiplication() {
        let mut parser =
            Parser::new([Token::NumLit(20.0), Token::Star, Token::NumLit(22.0)].into_iter());
        let target = Expr::Operation {
            left: Box::new(Expr::Literal(RuntimeVal::Number(20.0))),
            operation: Operator::Multiply,
            right: Box::new(Expr::Literal(RuntimeVal::Number(22.0))),
        };
        assert_eq!(parser.parse(), target);
    }
}
