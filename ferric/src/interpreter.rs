use std::{fmt::Display, io::Write};

use chrono::{DateTime, Utc};

use crate::parser::{BinaryOp, Expr, UnaryOp};
use std::thread;
use std::time::Duration;

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
    var_storage: Vec<RuntimeVal>,
    start_time: DateTime<Utc>,
}

impl<'a, W: Write> Interpreter<'a, W> {
    pub fn new(output: &'a mut W, var_storage_size: usize) -> Self {
        Self {
            output,
            var_storage: vec![RuntimeVal::Null; var_storage_size],
            start_time: Utc::now(),
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

    fn operation_modulo(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Number(n1 % n2),
            _ => panic!("Cannot take modulo of non-numbers"),
        }
    }

    fn unary_num_negate(&self, right: RuntimeVal) -> RuntimeVal {
        match right {
            RuntimeVal::Number(n) => RuntimeVal::Number(-n),
            _ => panic!("You can't not not negate that!"), // TODO : Update error messages
        }
    }

    fn operation_equal(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Boolean(n1 == n2),
            (RuntimeVal::String(_), RuntimeVal::String(_)) => todo!(),
            _ => {
                panic!("Tried to compare non-numbers!")
            }
        }
    }
    fn operation_neq(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Boolean(n1 != n2),
            (RuntimeVal::String(_), RuntimeVal::String(_)) => todo!(),
            _ => {
                panic!("Tried to compare non-numbers!")
            }
        }
    }
    fn operation_greater_than(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Boolean(n1 > n2),
            (RuntimeVal::String(_), RuntimeVal::String(_)) => todo!(),
            _ => {
                panic!("Tried to compare non-numbers!")
            }
        }
    }
    fn operation_less_than(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Boolean(n1 < n2),
            (RuntimeVal::String(_), RuntimeVal::String(_)) => todo!(),
            _ => {
                panic!("Tried to compare non-numbers!")
            }
        }
    }
    fn operation_geq(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Boolean(n1 >= n2),
            (RuntimeVal::String(_), RuntimeVal::String(_)) => todo!(),
            _ => {
                panic!("Tried to compare non-numbers!")
            }
        }
    }
    fn operation_leq(&self, left: RuntimeVal, right: RuntimeVal) -> RuntimeVal {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => RuntimeVal::Boolean(n1 <= n2),
            (RuntimeVal::String(_), RuntimeVal::String(_)) => todo!(),
            _ => {
                panic!("Tried to compare non-numbers!")
            }
        }
    }

    fn call_function(&mut self, func_name: RuntimeVal, args: Vec<RuntimeVal>) -> RuntimeVal {
        match func_name {
            RuntimeVal::Function(fn_name) => match fn_name.as_str() {
                "print" => builtin_print(self, args),
                "substr" => builtin_substr(self, args),
                "len" => builtin_len(self, args),
                "clock" => builtin_clock(self, args),
                "unix_time" => builtin_unix_time(self, args),
                "sleep" => builtin_sleep(self, args),
                _ => panic!("Function {} was not found", fn_name),
            },
            _ => panic!("Invalid function call"),
        }
    }

    fn unary_bit_not(&self, right: RuntimeVal) -> RuntimeVal {
        match right {
            RuntimeVal::Number(n) => {
                assert!(n.fract() == 0.0, "You can't bang a float!"); // TODO : Update Error messages
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
                    BinaryOp::Modulo => self.operation_modulo(left_val, right_val),
                    BinaryOp::Equal => self.operation_equal(left_val, right_val),
                    BinaryOp::NotEqual => self.operation_neq(left_val, right_val),
                    BinaryOp::GreaterThan => self.operation_greater_than(left_val, right_val),
                    BinaryOp::LessThan => self.operation_less_than(left_val, right_val),
                    BinaryOp::GreaterEq => self.operation_geq(left_val, right_val),
                    BinaryOp::LessEq => self.operation_leq(left_val, right_val),
                }
            }
            Expr::Unary { operation, right } => {
                let right_val = self.evaluate(right);
                match operation {
                    UnaryOp::Negate => self.unary_num_negate(right_val),
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
                let var = self
                    .var_storage
                    .get_mut(*slot)
                    .expect("Unable to fetch variable");
                *var = val;
                RuntimeVal::Null
            }
            Expr::VarGet { slot } => self.var_storage[*slot].clone(),
            Expr::VarSet { value, slot } => {
                self.var_storage[*slot] = self.evaluate(value);
                RuntimeVal::Null
            }
            Expr::If {
                cond,
                then,
                otherwise,
            } => {
                let eval_cond = self.evaluate(cond);
                match eval_cond {
                    RuntimeVal::Boolean(b) => {
                        if b {
                            self.evaluate(then)
                        } else if let Some(ow_branch) = otherwise {
                            self.evaluate(ow_branch)
                        } else {
                            RuntimeVal::Null
                        }
                    }
                    _ => {
                        panic!("Tried to read non-boolean expression as condition");
                    }
                }
            }
            Expr::Block(expressions) => {
                // returns Null on empty block
                let mut last_val = RuntimeVal::Null;
                for exp in expressions {
                    last_val = self.evaluate(exp);
                }
                last_val
            }
            Expr::While { cond, body } => {
                let RuntimeVal::Boolean(mut while_cond) = self.evaluate(cond) else {
                    panic!("While condition not boolean expression!");
                };

                while while_cond {
                    self.evaluate(body);

                    while_cond = match self.evaluate(cond) {
                        RuntimeVal::Boolean(b) => b,
                        _ => {
                            panic!("While condition not boolean expression!")
                        }
                    };
                }

                RuntimeVal::Null // temp, TODO: handle .collect()
            }
        }
    }

    pub fn interpret(&mut self, expressions: &Vec<Expr>) {
        for exp in expressions {
            self.evaluate(exp);
        }
    }
}

// BUILT-IN FUNCTIONS
// All of these must have type `fn(&mut Interpreter<'a, W>, Vec<RuntimeVal>) -> RuntimeVal`

fn builtin_print<W: Write>(i: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
    // TODO: How to do format strings here, at some point

    if args.is_empty() {
        writeln!(i.output).expect("Failed to write to output");
    } else {
        for val in args {
            write!(i.output, "{val}").expect("Failed to write to output"); // TODO: prints a function value
        }
        writeln!(i.output).expect("Failed to write to output");
    }
    RuntimeVal::Null
}

fn builtin_substr<W: Write>(i: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
    assert!(
        args.len() == 3,
        "substr(): Expect 3 args, got {}",
        args.len(),
    );
    if let RuntimeVal::String(string) = &args[0] {
        let start = expect_int(
            &args[1],
            format!(
                "substr(): Non-integer substring starting index: {}",
                &args[1]
            )
            .as_str(),
        );
        let end = expect_int(
            &args[2],
            format!("substr(): Non-integer substring ending index: {}", &args[2]).as_str(),
        );

        assert!(
            start >= 0 && end >= 0,
            "substr(): String indices cannot be negative"
        );
        assert!(
            start < string.len() as i64,
            "substr(): String starting index out of bounds: {}",
            start
        );
        assert!(
            end <= string.len() as i64,
            "substr(): String ending index out of bounds: {}",
            end
        );

        return RuntimeVal::String(string[(start as usize)..(end as usize)].to_string());
    }
    panic!(
        "substr(): Cannot take substring of non-string object: {}",
        &args[0]
    );
}

fn builtin_len<W: Write>(i: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
    assert!(
        args.len() == 1,
        "len(): Expect 1 argument, got {}",
        args.len()
    );
    match &args[0] {
        RuntimeVal::String(string) => RuntimeVal::Number(string.len() as f64),
        _ => panic!("len(): Object {} has no length property", &args[0]),
    }
}

fn builtin_clock<W: Write>(i: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
    assert!(
        args.is_empty(),
        "clock(): expected 0 args, got {}",
        args.len()
    );

    RuntimeVal::Number((Utc::now() - i.start_time).as_seconds_f64())
}

fn builtin_unix_time<W: Write>(i: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
    assert!(
        args.is_empty(),
        "unix_time: expected 0 args, got {}",
        args.len()
    );

    RuntimeVal::Number((Utc::now() - DateTime::UNIX_EPOCH).as_seconds_f64())
}

fn builtin_sleep<W: Write>(i: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
    assert!(
        args.len() == 1,
        "sleep(): expected 1 args, got {}",
        args.len()
    );
    match args[0] {
        RuntimeVal::Number(n) => {
            thread::sleep(Duration::from_secs_f64(n));
            RuntimeVal::Null
        }
        _ => panic!("Wrong arg type"),
    }
}

// HELPER FUNCTIONS
fn expect_int(val: &RuntimeVal, message: &str) -> i64 {
    match val {
        RuntimeVal::Number(num) => {
            assert!(num.fract() == 0.0, "{message}");
            *num as i64
        }
        _ => panic!("{message}"),
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::io::{sink, stdout};

    use crate::expr;

    use super::*;

    #[test]
    fn literal() {
        let mut out = stdout();

        let expr = Expr::Literal(RuntimeVal::Number(5.0));

        let mut interpreter = Interpreter::new(&mut out, 0);
        let res = interpreter.evaluate(&expr);

        assert_eq!(res, RuntimeVal::Number(5.0));
    }

    #[test]
    fn print_builtin() {
        let mut out = vec![];

        let expr = vec![
            expr!(Call(Ident("print"), [NumLit(4.0)])),
            expr!(Call(Ident("print"), [NumLit(5.0)])),
        ];

        let mut interpreter = Interpreter::new(&mut out, 0);
        interpreter.interpret(&expr);

        assert_eq!(out, b"4\n5\n");
    }

    #[test]
    fn substr_len_builtins() {
        let mut out = vec![];

        let expr = vec![expr!(Call(
            Ident("print"),
            [Call(
                Ident("substr"),
                [
                    StrLit("foo bar baz"),
                    NumLit(4.0),
                    Binary(
                        Call(Ident("len"), [StrLit("foo bar baz")]),
                        Subtract,
                        NumLit(4.0)
                    ),
                ]
            )]
        ))];

        let mut interpreter = Interpreter::new(&mut out, 0);
        interpreter.interpret(&expr);

        assert_eq!(out, b"bar\n");
    }

    #[test]
    fn binary_ops_int() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out, 0);

        let expr = expr!(Binary(NumLit(4.0), Add, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Number(9.0));

        let expr = expr!(Binary(NumLit(4.0), Subtract, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Number(-1.0));

        let expr = expr!(Binary(NumLit(4.0), Multiply, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Number(20.0));

        let expr = expr!(Binary(NumLit(4.0), Divide, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Number(0.8));

        let expr = expr!(Binary(NumLit(15.0), Modulo, NumLit(4.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Number(3.0));
    }

    #[test]
    fn binary_ops_bool() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out, 0);

        let expr = expr!(Binary(NumLit(4.0), GreaterThan, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Boolean(false));

        let expr = expr!(Binary(NumLit(4.0), LessThan, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Boolean(true));

        let expr = expr!(Binary(NumLit(4.0), Equal, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Boolean(false));

        let expr = expr!(Binary(NumLit(4.0), NotEqual, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Boolean(true));

        let expr = expr!(Binary(NumLit(4.0), GreaterEq, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Boolean(false));

        let expr = expr!(Binary(NumLit(4.0), LessEq, NumLit(5.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Boolean(true));
    }

    #[test]
    fn unary_int() {
        // bit not on numbers
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out, 0);

        let expr = expr!(Unary(BitNot, NumLit(0.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Number(-1.0));

        let expr = expr!(Unary(BitNot, NumLit(1.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Number(-2.0));

        let expr = expr!(Unary(BitNot, NumLit(-4.0)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Number(3.0));
    }

    /*
    #[test]
    fn unary_num_() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out, 0);

        let expr = expr!(Unary(Negate, BoolLit(true)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Boolean(false));

        let expr = expr!(Unary(Negate, BoolLit(false)));
        assert_eq!(interpreter.evaluate(&expr), RuntimeVal::Boolean(true));
    }
    */

    #[test]
    fn test_var_set() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out, 1);

        let expr = vec![expr!(Decl(NumLit(4.0), 0)), expr!(VarSet(NumLit(5.0), 0))];

        interpreter.interpret(&expr);

        assert_eq!(interpreter.var_storage[0], RuntimeVal::Number(5.0));
    }

    #[test]
    fn clock() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out, 1);

        let test = vec![
            expr!(Call(Ident("sleep"), [NumLit(1.0)])),
            expr!(Decl(Call(Ident("clock"), []), 0)),
        ];
        let eps = 0.012; // margin of time for program to run after sleep

        interpreter.interpret(&test);
        if let RuntimeVal::Number(c) = interpreter.var_storage[0] {
            assert!(
                eps > (c - 1.0).abs(),
                "clock ran longer than expected! {}",
                c - 1.0
            );
        } else {
            panic!("declared variable was not a number!");
        }
    }

    #[test]
    fn unix() {
        let mut out = sink();

        let eps = 1.0;
        let test = vec![expr!(Decl(Call(Ident("unix_time"), []), 0))];

        let mut interpreter = Interpreter::new(&mut out, 1);

        interpreter.interpret(&test);
        let RuntimeVal::Number(c) = interpreter.var_storage[0] else {
            panic!("invalid var in var_storage")
        };
        let elapsed = Utc::now() - (DateTime::UNIX_EPOCH);
        assert!(
            c - elapsed.as_seconds_f64() < eps,
            "unix time not reported correctly"
        );
    }
}
