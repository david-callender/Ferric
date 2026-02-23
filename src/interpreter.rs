use std::fmt::Display;

use crate::parser::{Expr, Operator};

#[derive(Clone)]
pub enum RuntimeVal {
    Number(f64),
    String(String),
}

impl Display for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
        }
    }
}

pub struct Interpreter {}



// evaluate runTime values together, break into serpate function

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }


    fn operation_add(&self,left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left,right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 + n2),
            (RuntimeVal::String(mut s1),RuntimeVal:: String(s2)) => {
                s1.push_str(&s2);
                RuntimeVal::String(s1)
            },
            _ => panic!("You can't add those, idiot!") // TODO: Update error messages
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] fn operation_multiply(&self,left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left,right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 * n2),
            (RuntimeVal::String(mut s1), RuntimeVal::Number(n)) => {
                assert!(n.fract() != 0.0, "You can't multiply a string by a float!");

                s1 = s1.repeat(n as usize);

                RuntimeVal::String(s1)
            },
            _ => panic!("You can't multiply those, idiot!") // TODO: Update error messages
        }
    }

    fn operation_subtract(&self,left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left,right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 - n2),
            _ => panic!("You can't subtract those, idiot!") // TODO: Update error messages
        }
    }

    fn operation_divide(&self,left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left,right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 / n2),
            _ => panic!("You can't divide those, idiot!") // TODO: Update error messages
        }
    }

    // evalute -> condense tree -> runTimeVal
    fn evaluate(&self, expr: &Expr) -> RuntimeVal {
        match expr {
            Expr::Literal(runtime_val) => runtime_val.clone(),
            Expr::Operation { left, operation, right } => {
                let left_val = self.evaluate(left);
                let right_val = self.evaluate(right);
                match operation {
                    Operator::Add => self.operation_add(left_val, right_val),
                    Operator::Subtract => self.operation_subtract(left_val, right_val),
                    Operator::Multiply => self.operation_multiply(left_val, right_val),
                    Operator::Divide => self.operation_divide(left_val, right_val)
                }
            },
        }
    }

    // interpret -> calls evaluate, prints runTimeVal
    pub fn interpret(&self, expr: Expr) {
        let finalVal = self.evaluate(&expr);
        
        // matches nodes in the ast
        match finalVal {
            RuntimeVal::Number(n) => println!("{n}"),
            RuntimeVal::String(s) => println!("{s}")
        }
    }


    
}
