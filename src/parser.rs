use crate::lexer::Token;

pub enum Stmt {}

pub struct Parser {}

impl Parser {
    pub fn new(_tokens: Vec<Token>) -> Self {
        Self {}
    }

    pub fn parse(&self) -> Vec<Stmt> {
        vec![]
    }
}
