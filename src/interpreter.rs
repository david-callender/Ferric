use std::{fmt::Display, io::Write};

use crate::parser::{BinaryOp, Expr, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeVal {
    Number(f64),
    String(String),
    Boolean(bool),
    Function(String),
    Null,
}

impl Display for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Function(n) => write!(f, "{n}"),
            Self::Null => write!(f, "Null"),
        }
    }
}

pub struct Interpreter<'a, W: Write> {
    output: &'a mut W,
    var_stack: Vec<RuntimeVal>,
}

impl<'a, W: Write> Interpreter<'a, W> {
    pub fn new(output: &'a mut W, var_stack_size: usize) -> Self {
        Self {
            output,
            var_stack: Vec::with_capacity(var_stack_size),
        }
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
            _ => panic!("You can't not not negate that!"), // TODO : Update error messages
        }
    }

    fn call_function(&mut self, func_name: RuntimeVal, args: Vec<RuntimeVal>) -> RuntimeVal {
        match func_name {
            RuntimeVal::Function(fn_name) => {
                match fn_name.as_str() {
                    "print" => {
                        // TODO: How to do format strings here, at some point

                        if args.is_empty() {
                            writeln!(self.output).expect("Failed to write to output");
                        } else {
                            for val in args {
                                writeln!(self.output, "{val}").expect("Failed to write to output"); // TODO: prints a function value
                            }
                        }
                        RuntimeVal::Null
                    }
                    _ => panic!(""),
                }
            }
            _ => panic!("Invalid function call"),
        }
    }

    fn unary_bit_not(&self, right: RuntimeVal) -> RuntimeVal {
        match right {
            RuntimeVal::Number(n) => {
                assert!(n.fract() != 0.0, "You can't bang a float!"); // TODO : Update Error messages
                RuntimeVal::Number(!(n as i64) as f64)
            }
            _ => panic!("You can't not not negate that (number)!"), // TODO : Update error messages
        }
    }

    // evalute -> condense tree -> runTimeVal
    fn evaluate(&mut self, expr: &Expr) -> RuntimeVal {
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
            }
            Expr::Unary { operation, right } => {
                let right_val = self.evaluate(right);
                match operation {
                    UnaryOp::Negate => self.unary_negate(right_val),
                    UnaryOp::BitNot => self.unary_bit_not(right_val),
                }
            }
            Expr::Call { callee, args } => {
                let func_caller = self.evaluate(callee);
                let args = args.iter().map(|expr| self.evaluate(expr)).collect();

                self.call_function(func_caller, args)
            }
            Expr::Ident(name) => {
                // check if function exists
                RuntimeVal::Function(name.clone())
            }
            Expr::Decl { value, slot } => {
                let val = self.evaluate(value);
                self.var_stack.insert(*slot, val);
                RuntimeVal::Null
            }
            Expr::VarGet { slot } => self.var_stack[*slot].clone(),
        }
    }

    pub fn interpret(&mut self, expressions: &Vec<Expr>) {
        // expr - head of ast tree
        // prints out the RuntimeVal of expr

        for exp in expressions {
            self.evaluate(exp);
        }
    }
}
