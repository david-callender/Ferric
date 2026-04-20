use std::cell::RefCell;
use std::rc::Rc;
use std::{fmt::Display, io::Write};

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::parser::{BinaryOp, Expr, ExprKind, UnaryOp};
use std::thread;
use std::time::Duration;

#[derive(Debug, Error, Clone)]
pub enum RuntimeError {
    #[error("binary type mismatch")]
    BinaryTypeMismatch,

    #[error("unary type mismatch")]
    UnaryTypeMismatch,
}

// Module error type
type Res<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    BuiltIn(String),
    Custom {
        param_count: usize,
        body: Rc<[Expr]>,
        closure: Environment,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeVal {
    Number(f64),
    String(String),
    Boolean(bool),
    Function(Function),
    Null,
}

impl Display for RuntimeVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Function(_) => write!(f, "function"),
            Self::Null => write!(f, "Null"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct EnvLevel {
    parent: Option<Rc<RefCell<EnvLevel>>>,
    vars: Vec<RuntimeVal>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment(Rc<RefCell<EnvLevel>>);

impl Environment {
    fn new_global() -> Self {
        Self(Rc::new(RefCell::new(EnvLevel {
            parent: None,
            vars: vec![],
        })))
    }
    fn new(parent: Self) -> Self {
        Self(Rc::new(RefCell::new(EnvLevel {
            parent: Some(parent.0),
            vars: vec![],
        })))
    }

    fn decendent(&self, times: usize) -> Self {
        let mut env = self.0.clone();
        for _ in 0..times {
            let a = env
                .borrow()
                .parent
                .clone()
                .expect("depth was too large for current env");
            env = a;
        }

        Self(env)
    }

    fn get(&self, depth: usize, slot: usize) -> RuntimeVal {
        let desc = self.decendent(depth);
        let frame = desc.0.borrow();
        frame
            .vars
            .get(slot)
            .unwrap_or_else(|| panic!("depth: {depth}, slot: {slot}"))
            .clone()
    }

    fn declare(&self, value: RuntimeVal) {
        self.0.borrow_mut().vars.push(value);
    }

    fn set(&self, depth: usize, slot: usize, value: RuntimeVal) {
        *self
            .decendent(depth)
            .0
            .borrow_mut()
            .vars
            .get_mut(slot)
            .unwrap() = value;
    }
}

fn operation_add(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 + n2)),
        (RuntimeVal::String(mut s1), RuntimeVal::String(s2)) => {
            s1.push_str(&s2);
            Ok(RuntimeVal::String(s1))
        }
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

fn operation_multiply(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 * n2)),
        (RuntimeVal::String(mut s1), RuntimeVal::Number(n)) => {
            assert!(n.fract() != 0.0, "You can't multiply a string by a float!");

            s1 = s1.repeat(n as usize);

            Ok(RuntimeVal::String(s1))
        }
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

fn operation_subtract(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 - n2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

fn operation_divide(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 / n2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

fn operation_modulo(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 % n2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn unary_num_negate(right: RuntimeVal) -> Res<RuntimeVal> {
    match right {
        RuntimeVal::Number(n) => Ok(RuntimeVal::Number(-n)),
        _ => Err(RuntimeError::UnaryTypeMismatch),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn unary_bool_not(right: RuntimeVal) -> Res<RuntimeVal> {
    match right {
        RuntimeVal::Boolean(b) => Ok(RuntimeVal::Boolean(!b)),
        _ => Err(RuntimeError::UnaryTypeMismatch),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn unary_bit_not(right: RuntimeVal) -> Res<RuntimeVal> {
    match right {
        RuntimeVal::Number(n) => {
            assert!(n.fract() == 0.0, "You can't bang a float!"); // TODO : Update Error messages
            Ok(RuntimeVal::Number(!(n as i64) as f64))
        }
        _ => Err(RuntimeError::UnaryTypeMismatch),
    }
}

#[allow(clippy::float_cmp)]
fn operation_equal(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 == n2)),
        (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 == s2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

#[allow(clippy::float_cmp)]
fn operation_neq(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 != n2)),
        (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 != s2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

fn operation_greater_than(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 > n2)),
        (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 > s2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

fn operation_less_than(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 < n2)),
        (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 < s2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

fn operation_geq(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 >= n2)),
        (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 >= s2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

fn operation_leq(left: RuntimeVal, right: RuntimeVal) -> Res<RuntimeVal> {
    match (left, right) {
        (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 <= n2)),
        (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 <= s2)),
        _ => Err(RuntimeError::BinaryTypeMismatch),
    }
}

pub struct Interpreter<'a, W: Write> {
    output: &'a mut W,
    env: Environment,
    start_time: DateTime<Utc>,
}

impl<'a, W: Write> Interpreter<'a, W> {
    pub fn new(output: &'a mut W) -> Self {
        Self {
            output,
            env: Environment::new_global(),
            start_time: Utc::now(),
        }
    }

    fn call_function(&mut self, fn_obj: RuntimeVal, args: Vec<RuntimeVal>) -> Res<RuntimeVal> {
        Ok(match fn_obj {
            RuntimeVal::Function(Function::BuiltIn(fn_name)) => match fn_name.as_str() {
                "print" => builtin_print(self, args),
                "substr" => builtin_substr(self, args),
                "len" => builtin_len(self, args),
                "clock" => builtin_clock(self, args),
                "unix_time" => builtin_unix_time(self, args),
                "sleep" => builtin_sleep(self, args),
                x => panic!("Function {x} was not found"),
            },
            RuntimeVal::Function(Function::Custom {
                param_count,
                body,
                closure,
            }) => {
                assert!(param_count == args.len());
                let func_env = Environment::new(closure);

                for arg in args {
                    func_env.declare(arg);
                }

                self.evaluate_block(body.as_ref(), func_env)?
            }
            _ => panic!("Invalid function call"),
        })
    }

    fn evaluate_block(&mut self, expressions: &[Expr], new_env: Environment) -> Res<RuntimeVal> {
        let old_env = self.env.clone();
        self.env = new_env;
        // returns Null on empty block
        let mut last_val = RuntimeVal::Null;
        for exp in expressions {
            last_val = self.evaluate(exp)?;
        }
        self.env = old_env;
        Ok(last_val)
    }

    // evalute -> condense tree -> runTimeVal
    fn evaluate(&mut self, expr: &Expr) -> Res<RuntimeVal> {
        Ok(match &expr.kind {
            ExprKind::Literal(runtime_val) => runtime_val.clone(),
            ExprKind::Binary {
                left,
                operation,
                right,
            } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                match operation {
                    BinaryOp::Add => operation_add(left_val, right_val)?,
                    BinaryOp::Subtract => operation_subtract(left_val, right_val)?,
                    BinaryOp::Multiply => operation_multiply(left_val, right_val)?,
                    BinaryOp::Divide => operation_divide(left_val, right_val)?,
                    BinaryOp::Modulo => operation_modulo(left_val, right_val)?,
                    BinaryOp::Equal => operation_equal(left_val, right_val)?,
                    BinaryOp::NotEqual => operation_neq(left_val, right_val)?,
                    BinaryOp::GreaterThan => operation_greater_than(left_val, right_val)?,
                    BinaryOp::LessThan => operation_less_than(left_val, right_val)?,
                    BinaryOp::GreaterEq => operation_geq(left_val, right_val)?,
                    BinaryOp::LessEq => operation_leq(left_val, right_val)?,
                }
            }
            ExprKind::Unary { operation, right } => {
                let right_val = self.evaluate(right)?;
                match operation {
                    UnaryOp::Negate => unary_num_negate(right_val)?,
                    UnaryOp::BitNot => unary_bit_not(right_val)?,
                    UnaryOp::BoolNot => unary_bool_not(right_val)?,
                }
            }
            ExprKind::Call { callee, args } => {
                let func_caller = self.evaluate(callee)?;
                let args = args
                    .iter()
                    .map(|expr| self.evaluate(expr))
                    .collect::<Res<Vec<RuntimeVal>>>()?;

                self.call_function(func_caller, args)?
            }
            ExprKind::Ident(name) => {
                // check if function exists
                RuntimeVal::Function(Function::BuiltIn(name.clone()))
            }
            ExprKind::Decl { value } => {
                let val = self.evaluate(value)?;
                self.env.declare(val);

                RuntimeVal::Null
            }
            ExprKind::VarGet { depth, slot } => self.env.get(*depth, *slot),
            ExprKind::VarSet { value, depth, slot } => {
                let val = self.evaluate(value)?;
                self.env.set(*depth, *slot, val);
                RuntimeVal::Null
            }
            ExprKind::Func { param_count, body } => {
                RuntimeVal::Function(Function::Custom {
                    param_count: *param_count,
                    body: body.to_owned().into(), // body.to_owned().into(),
                    closure: self.env.clone(),
                })
            }
            ExprKind::If {
                cond,
                then,
                otherwise,
            } => {
                let eval_cond = self.evaluate(cond)?;
                match eval_cond {
                    RuntimeVal::Boolean(b) => {
                        if b {
                            self.evaluate_block(then, Environment::new(self.env.clone()))?
                        } else if let Some(ow_branch) = otherwise {
                            self.evaluate(ow_branch)?
                        } else {
                            RuntimeVal::Null
                        }
                    }
                    _ => {
                        panic!("Tried to read non-boolean expression as condition");
                    }
                }
            }
            ExprKind::Block(expressions) => {
                self.evaluate_block(expressions, Environment::new(self.env.clone()))?
            }
            ExprKind::While { cond, body } => {
                let RuntimeVal::Boolean(mut while_cond) = self.evaluate(cond)? else {
                    panic!("While condition not boolean expression!");
                };

                while while_cond {
                    self.evaluate_block(body, Environment::new(self.env.clone()))?;

                    while_cond = match self.evaluate(cond)? {
                        RuntimeVal::Boolean(b) => b,
                        _ => {
                            panic!("While condition not boolean expression!")
                        }
                    };
                }

                RuntimeVal::Null // temp, TODO: handle .collect()
            }
        })
    }

    pub fn interpret(&mut self, expressions: &Vec<Expr>) -> Res<()> {
        for exp in expressions {
            self.evaluate(exp)?;
        }
        Ok(())
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

#[allow(clippy::needless_pass_by_value)]
fn builtin_substr<W: Write>(_: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
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
            "substr(): String starting index out of bounds: {start}"
        );
        assert!(
            end <= string.len() as i64,
            "substr(): String ending index out of bounds: {end}"
        );

        return RuntimeVal::String(string[(start as usize)..(end as usize)].to_string());
    }
    panic!(
        "substr(): Cannot take substring of non-string object: {}",
        args[0]
    );
}

#[allow(clippy::needless_pass_by_value)]
fn builtin_len<W: Write>(_: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
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

#[allow(clippy::needless_pass_by_value)]
fn builtin_clock<W: Write>(i: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
    assert!(
        args.is_empty(),
        "clock(): expected 0 args, got {}",
        args.len()
    );

    RuntimeVal::Number((Utc::now() - i.start_time).as_seconds_f64())
}

#[allow(clippy::needless_pass_by_value)]
fn builtin_unix_time<W: Write>(_: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
    assert!(
        args.is_empty(),
        "unix_time: expected 0 args, got {}",
        args.len()
    );

    RuntimeVal::Number((Utc::now() - DateTime::UNIX_EPOCH).as_seconds_f64())
}

#[allow(clippy::needless_pass_by_value)]
fn builtin_sleep<W: Write>(_: &mut Interpreter<'_, W>, args: Vec<RuntimeVal>) -> RuntimeVal {
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

    use crate::{expr, loc::{Loc, Span}, parser::ExprKind};

    use super::*;

    #[test]
    fn literal() {
        let mut out = stdout();

        let expr = expr!(NumLit(5.0));

        let mut interpreter = Interpreter::new(&mut out);
        let res = interpreter.evaluate(&expr).unwrap();

        assert_eq!(res, RuntimeVal::Number(5.0));
    }

    #[test]
    fn print_builtin() {
        let mut out = vec![];

        let expr = vec![
            expr!(Call(Ident("print"), [NumLit(4.0)])),
            expr!(Call(Ident("print"), [NumLit(5.0)])),
        ];

        let mut interpreter = Interpreter::new(&mut out);
        interpreter.interpret(&expr).unwrap();

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

        let mut interpreter = Interpreter::new(&mut out);
        interpreter.interpret(&expr).unwrap();

        assert_eq!(out, b"bar\n");
    }

    #[test]
    fn binary_ops_int() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out);

        let expr = expr!(Binary(NumLit(4.0), Add, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(9.0)
        );

        let expr = expr!(Binary(NumLit(4.0), Subtract, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(-1.0)
        );

        let expr = expr!(Binary(NumLit(4.0), Multiply, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(20.0)
        );

        let expr = expr!(Binary(NumLit(4.0), Divide, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(0.8)
        );

        let expr = expr!(Binary(NumLit(15.0), Modulo, NumLit(4.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(3.0)
        );
    }

    #[test]
    fn binary_ops_bool() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out);

        let expr = expr!(Binary(NumLit(4.0), GreaterThan, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Boolean(false)
        );

        let expr = expr!(Binary(NumLit(4.0), LessThan, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Boolean(true)
        );

        let expr = expr!(Binary(NumLit(4.0), Equal, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Boolean(false)
        );

        let expr = expr!(Binary(NumLit(4.0), NotEqual, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Boolean(true)
        );

        let expr = expr!(Binary(NumLit(4.0), GreaterEq, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Boolean(false)
        );

        let expr = expr!(Binary(NumLit(4.0), LessEq, NumLit(5.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Boolean(true)
        );
    }

    #[test]
    fn unary_int() {
        // bit not on numbers
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out);

        let expr = expr!(Unary(BitNot, NumLit(0.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(-1.0)
        );

        let expr = expr!(Unary(BitNot, NumLit(1.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(-2.0)
        );

        let expr = expr!(Unary(BitNot, NumLit(-4.0)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(3.0)
        );
    }

    #[test]
    fn unary_num_() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out);

        let expr = expr!(Unary(BoolNot, BoolLit(true)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Boolean(false)
        );

        let expr = expr!(Unary(BoolNot, BoolLit(false)));
        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Boolean(true)
        );
    }

    #[test]
    fn test_var_set() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out);

        let expr = expr!(Block {
            Decl(NumLit(4.0)),
            VarSet(NumLit(5.0), 0, 0),
            VarGet(0, 0)
        });

        assert_eq!(
            interpreter.evaluate(&expr).unwrap(),
            RuntimeVal::Number(5.0)
        );
    }

    #[test]
    fn clock() {
        let mut out = sink();
        let mut interpreter = Interpreter::new(&mut out);

        let expr = expr!(Block {
            Call(Ident("sleep"), [NumLit(1.0)]),
            Decl(Call(Ident("clock"), [])),
            VarGet(0, 0),
        });

        let eps = 0.1; // margin of time for program to run after sleep

        let RuntimeVal::Number(c) = interpreter.evaluate(&expr).unwrap() else {
            panic!("invalid type returned")
        };
        assert!(
            eps > (c - 1.0).abs(),
            "sleep ran {}s longer than expected",
            c - 1.0
        );
    }

    #[test]
    fn unix() {
        let mut out = sink();

        let eps = 1.0;
        let expr = expr!(Call(Ident("unix_time"), []));

        let mut interpreter = Interpreter::new(&mut out);

        interpreter.evaluate(&expr).unwrap();
        let RuntimeVal::Number(c) = interpreter.evaluate(&expr).unwrap() else {
            panic!("invalid type returned")
        };
        let elapsed = Utc::now() - (DateTime::UNIX_EPOCH);
        assert!(
            c - elapsed.as_seconds_f64() < eps,
            "unix time not reported correctly (delta: {})",
            c - elapsed.as_seconds_f64()
        );
    }
}
