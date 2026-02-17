use std::fmt::Display;

use crate::parser::Expr;

pub enum RuntimeVal {
    Number(f64),
    String(String),
}

impl Display for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) =>  write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}")
        }
        
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, expr: Expr) {
        match expr {
            Expr::Literal(l) => print!("{l}")
        }
       
    }
}
