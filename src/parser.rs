use std::iter::Peekable;

use crate::{interpreter::RuntimeVal, lexer::Token};

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

    pub fn parse(&mut self) -> Expr {
        todo!();
    }

    pub fn parse_basic(&mut self) -> Expr {
        let token = self.stream.next().expect("expected basic token, got none");
        match token {
            Token::StringLit(string) => Expr::Literal(RuntimeVal::String(string)),
            Token::NumLit(number) => Expr::Literal(RuntimeVal::Number(number)),
            _ => panic!("expected basic token, got non-basic token"),
        }
    }
}
