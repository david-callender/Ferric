use crate::parser::Expr;

pub enum RuntimeVal {
    Number(f64),
    String(String),
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, expr: Expr) {}
}
