use std::fmt::Display;

use crate::parser::{BinaryOp, Expr, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeVal {
    Number(f64),
    String(String),
    Boolean(bool)
}

impl Display for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
        }
    }
}

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    fn operation_add(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 + n2),
            (RuntimeVal::String(mut s1), RuntimeVal::String(s2)) => {
                s1.push_str(&s2);
                RuntimeVal::String(s1)
            }
            _ => panic!("You can't add those, idiot!"), // TODO: Update error messages
        }
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn operation_multiply(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 * n2),
            (RuntimeVal::String(mut s1), RuntimeVal::Number(n)) => {
                assert!(n.fract() != 0.0, "You can't multiply a string by a float!");

                s1 = s1.repeat(n as usize);

                RuntimeVal::String(s1)
            }
            _ => panic!("You can't multiply those, idiot!"), // TODO: Update error messages
        }
    }

    fn operation_subtract(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 - n2),
            _ => panic!("You can't subtract those, idiot!"), // TODO: Update error messages
        }
    }

    fn operation_divide(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 / n2),
            _ => panic!("You can't divide those, idiot!"), // TODO: Update error messages
        }
    }

    fn unary_negate(&self, right: RuntimeVal) -> RuntimeVal {
        match right {
            RuntimeVal::Boolean(b) => RuntimeVal::Boolean(!b),
            _ => panic!("You can't not not negate that!") // TODO : Update error messages
        }
    }

    fn unary_bit_not(&self, right: RuntimeVal) -> RuntimeVal {
        match right {
            RuntimeVal::Number(n) => {
                assert!(n.fract() != 0.0, "You can't bang a float!"); // TODO : Update Error messages
                RuntimeVal::Number(!(n as i64) as f64)
            },
            _ => panic!("You can't not not negate that (number)!") // TODO : Update error messages
        }
    }

    // evalute -> condense tree -> runTimeVal
    fn evaluate(&self, expr: &Expr) -> RuntimeVal {
        match expr {
            Expr::Literal(runtime_val) => runtime_val.clone(),
            Expr::Binary {
                left,
                operation,
                right,
            } => {
                let left_val = self.evaluate(left);
                let right_val = self.evaluate(right);
                match operation {
                    BinaryOp::Add => self.operation_add(left_val, right_val),
                    BinaryOp::Subtract => self.operation_subtract(left_val, right_val),
                    BinaryOp::Multiply => self.operation_multiply(left_val, right_val),
                    BinaryOp::Divide => self.operation_divide(left_val, right_val),
                }
            },
            Expr::Unary {
                operation, right 
            } => {
                let right_val = self.evaluate(right);
                match operation {
                    UnaryOp::Negate => self.unary_negate(right_val),
                    UnaryOp::BitNot => self.unary_bit_not(right_val),
                }
            }
            
        }
    }

    pub fn interpret(&self, expr: Expr) {
        // expr - head of ast tree
        // prints out the RuntimeVal of expr
        let final_val = self.evaluate(&expr);

        match final_val {
            RuntimeVal::Number(n) => println!("{n}"),
            RuntimeVal::String(s) => println!("{s}"),
            RuntimeVal::Boolean(b) => println!("{b}")
        }
    }
}
