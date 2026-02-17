use std::iter::Peekable;

use crate::{interpreter::RuntimeVal, lexer::Token};

pub enum Expr {
    Literal(RuntimeVal)
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
}
