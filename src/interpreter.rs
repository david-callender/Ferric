use crate::parser::Stmt;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, _stmts: Vec<Stmt>) {}
}
