use std::{
    cell::RefCell, collections::VecDeque, fmt::Display, io::Write, rc::Rc, thread, time::Duration,
};

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{
    loc::{ProgramSrc, ProgramSrcInner, Span},
    parser::{BinaryOp, Expr, ExprKind, UnaryOp},
};

#[derive(Debug, Error, Clone)]
pub enum RuntimeError {
    #[error("Binary type mismatch between {left:#} and {right:#}\n{}", .span.format(src, &format!("cannot {op} {left:#} and {right:#}")))]
    BinaryTypeMismatch {
        src: ProgramSrc,
        span: Span,
        op: BinaryOp,
        left: RuntimeVal,
        right: RuntimeVal,
    },

    #[error("Unary type mismatch with {right}\n{}", .span.format(src, &format!("cannot {op} {right}")))]
    UnaryTypeMismatch {
        src: ProgramSrc,
        span: Span,
        op: UnaryOp,
        right: RuntimeVal,
    },

    #[error("Expected an integer\n{}", .span.format(src, message))]
    ExpectedInt {
        src: ProgramSrc,
        span: Span,
        message: &'static str,
    },

    #[error("Invalid function object\n{}", .span.format(src, &format!("this object of type {obj:#} is not callable")))]
    InvalidFunctionObject {
        src: ProgramSrc,
        span: Span,
        obj: RuntimeVal,
    },

    #[error("Invalid builtin function\n{}", .span.format(src, "this is not a valid builtin function"))]
    InvalidBuiltinFunction { src: ProgramSrc, span: Span },

    #[error("Invalid condition\n{}", .span.format(src, &format!("this condition must be a boolean, but it is a {actual:#}")))]
    InvalidCondition {
        src: ProgramSrc,
        span: Span,
        actual: RuntimeVal,
    },
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
        if f.alternate() {
            match self {
                Self::Number(_) => write!(f, "number"),
                Self::String(_) => write!(f, "string"),
                Self::Boolean(_) => write!(f, "boolean"),
                Self::Function(_) => write!(f, "function"),
                Self::Null => write!(f, "null"),
            }
        } else {
            match self {
                Self::Number(n) => write!(f, "{n}"),
                Self::String(s) => write!(f, "{s}"),
                Self::Boolean(b) => write!(f, "{b}"),
                Self::Function(_) => write!(f, "function"),
                Self::Null => write!(f, "Null"),
            }
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

pub struct Interpreter<'a, W: Write> {
    src: ProgramSrc,
    output: &'a mut W,
    env: Environment,
    start_time: DateTime<Utc>,
}

impl<'a, W: Write> Interpreter<'a, W> {
    pub fn new(src: ProgramSrc, output: &'a mut W) -> Self {
        Self {
            src,
            output,
            env: Environment::new_global(),
            start_time: Utc::now(),
        }
    }

    pub fn test(output: &'a mut W) -> Self {
        Self {
            src: Rc::new(ProgramSrcInner::new(String::new())),
            output,
            env: Environment::new_global(),
            start_time: Utc::now(),
        }
    }

    fn operation_add(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 + n2)),
            (RuntimeVal::String(mut s1), RuntimeVal::String(s2)) => {
                s1.push_str(&s2);
                Ok(RuntimeVal::String(s1))
            }
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::Add,
                left,
                right,
            }),
        }
    }

    fn operation_multiply(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 * n2)),
            (RuntimeVal::String(mut s1), RuntimeVal::Number(n)) => {
                if n.fract() != 0.0 {
                    return Err(RuntimeError::ExpectedInt {
                        src: self.src.clone(),
                        span: s,
                        message: "string repeat count must be an integer",
                    });
                }

                s1 = s1.repeat(n as usize);

                Ok(RuntimeVal::String(s1))
            }
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::Multiply,
                left,
                right,
            }),
        }
    }

    fn operation_subtract(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 - n2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::Subtract,
                left,
                right,
            }),
        }
    }

    fn operation_divide(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 / n2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::Divide,
                left,
                right,
            }),
        }
    }

    fn operation_modulo(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Number(n1 % n2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::Modulo,
                left,
                right,
            }),
        }
    }

    #[allow(clippy::float_cmp)]
    fn operation_equal(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 == n2)),
            (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 == s2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::Equal,
                left,
                right,
            }),
        }
    }

    #[allow(clippy::float_cmp)]
    fn operation_neq(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 != n2)),
            (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 != s2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::NotEqual,
                left,
                right,
            }),
        }
    }

    fn operation_greater_than(
        &self,
        left: RuntimeVal,
        right: RuntimeVal,
        s: Span,
    ) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 > n2)),
            (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 > s2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::GreaterThan,
                left,
                right,
            }),
        }
    }

    fn operation_less_than(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 < n2)),
            (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 < s2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::LessThan,
                left,
                right,
            }),
        }
    }

    fn operation_geq(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 >= n2)),
            (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 >= s2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::GreaterEq,
                left,
                right,
            }),
        }
    }

    fn operation_leq(&self, left: RuntimeVal, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match (left, right) {
            (RuntimeVal::Number(n1), RuntimeVal::Number(n2)) => Ok(RuntimeVal::Boolean(n1 <= n2)),
            (RuntimeVal::String(s1), RuntimeVal::String(s2)) => Ok(RuntimeVal::Boolean(s1 <= s2)),
            (left, right) => Err(RuntimeError::BinaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: BinaryOp::LessEq,
                left,
                right,
            }),
        }
    }

    fn unary_num_negate(&self, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match right {
            RuntimeVal::Number(n) => Ok(RuntimeVal::Number(-n)),
            _ => Err(RuntimeError::UnaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: UnaryOp::Negate,
                right,
            }),
        }
    }

    fn unary_bool_not(&self, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match right {
            RuntimeVal::Boolean(b) => Ok(RuntimeVal::Boolean(!b)),
            _ => Err(RuntimeError::UnaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: UnaryOp::BoolNot,
                right,
            }),
        }
    }

    fn unary_bit_not(&self, right: RuntimeVal, s: Span) -> Res<RuntimeVal> {
        match right {
            RuntimeVal::Number(n) => {
                if n.fract() != 0.0 {
                    return Err(RuntimeError::ExpectedInt {
                        src: self.src.clone(),
                        span: s,
                        message: "unary bit not expects an integer",
                    });
                }
                Ok(RuntimeVal::Number(!(n as i64) as f64))
            }
            _ => Err(RuntimeError::UnaryTypeMismatch {
                src: self.src.clone(),
                span: s,
                op: UnaryOp::BitNot,
                right,
            }),
        }
    }

    fn binary(&mut self, left: &Expr, op: BinaryOp, right: &Expr, span: Span) -> Res<RuntimeVal> {
        let left_val = self.evaluate(left)?;
        let right_val = self.evaluate(right)?;
        Ok(match op {
            BinaryOp::Add => self.operation_add(left_val, right_val, span)?,
            BinaryOp::Subtract => self.operation_subtract(left_val, right_val, span)?,
            BinaryOp::Multiply => self.operation_multiply(left_val, right_val, span)?,
            BinaryOp::Divide => self.operation_divide(left_val, right_val, span)?,
            BinaryOp::Modulo => self.operation_modulo(left_val, right_val, span)?,
            BinaryOp::Equal => self.operation_equal(left_val, right_val, span)?,
            BinaryOp::NotEqual => self.operation_neq(left_val, right_val, span)?,
            BinaryOp::GreaterThan => self.operation_greater_than(left_val, right_val, span)?,
            BinaryOp::LessThan => self.operation_less_than(left_val, right_val, span)?,
            BinaryOp::GreaterEq => self.operation_geq(left_val, right_val, span)?,
            BinaryOp::LessEq => self.operation_leq(left_val, right_val, span)?,
        })
    }

    fn unary(&mut self, op: UnaryOp, right: &Expr, expr: &Expr) -> Res<RuntimeVal> {
        let right_val = self.evaluate(right)?;
        Ok(match op {
            UnaryOp::Negate => self.unary_num_negate(right_val, expr.span)?,
            UnaryOp::BitNot => self.unary_bit_not(right_val, expr.span)?,
            UnaryOp::BoolNot => self.unary_bool_not(right_val, expr.span)?,
        })
    }

    fn call_function(&mut self, f: RuntimeVal, args: Vec<RuntimeVal>, s: Span) -> Res<RuntimeVal> {
        Ok(match f {
            RuntimeVal::Function(Function::BuiltIn(fn_name)) => {
                let args = Args(args.into());
                match fn_name.as_str() {
                    "print" => builtin_print(self, args),
                    "substr" => builtin_substr(args),
                    "len" => builtin_len(args),
                    "clock" => builtin_clock(self, args),
                    "unix_time" => builtin_unix_time(args),
                    "sleep" => builtin_sleep(args),
                    _ => {
                        return Err(RuntimeError::InvalidBuiltinFunction {
                            src: self.src.clone(),
                            span: s,
                        });
                    }
                }
            }
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
            obj => {
                return Err(RuntimeError::InvalidFunctionObject {
                    src: self.src.clone(),
                    span: s,
                    obj,
                });
            }
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
            ExprKind::Binary { left, op, right } => self.binary(left, *op, right, expr.span)?,
            ExprKind::Unary { op, right } => self.unary(*op, right, expr)?,
            ExprKind::Call { callee, args } => {
                let func_caller = self.evaluate(callee)?;
                let args = args
                    .iter()
                    .map(|expr| self.evaluate(expr))
                    .collect::<Res<Vec<RuntimeVal>>>()?;

                self.call_function(func_caller, args, expr.span)?
            }
            ExprKind::Ident(name) => RuntimeVal::Function(Function::BuiltIn(name.clone())),
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
                    actual => {
                        return Err(RuntimeError::InvalidCondition {
                            src: self.src.clone(),
                            span: expr.span,
                            actual,
                        });
                    }
                }
            }
            ExprKind::Block(expressions) => {
                self.evaluate_block(expressions, Environment::new(self.env.clone()))?
            }
            ExprKind::While { cond, body } => {
                while {
                    match self.evaluate(cond)? {
                        RuntimeVal::Boolean(b) => b,
                        actual => {
                            return Err(RuntimeError::InvalidCondition {
                                src: self.src.clone(),
                                span: expr.span,
                                actual,
                            });
                        }
                    }
                } {
                    self.evaluate_block(body, Environment::new(self.env.clone()))?;
                }

                RuntimeVal::Null
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

#[derive(Debug, Clone)]
struct Args(VecDeque<RuntimeVal>);

impl Args {
    fn next_number(&mut self) -> f64 {
        match self.0.pop_front().unwrap() {
            RuntimeVal::Number(n) => n,
            _ => panic!(),
        }
    }

    fn next_int(&mut self) -> i64 {
        match self.0.pop_front().unwrap() {
            RuntimeVal::Number(n) if n.fract() == 0.0 => n as i64,
            _ => panic!(),
        }
    }

    fn next_string(&mut self) -> String {
        match self.0.pop_front().unwrap() {
            RuntimeVal::String(s) => s,
            _ => panic!(),
        }
    }

    // fn next_bool(&mut self) -> bool {
    //     match self.0.pop_front().unwrap() {
    //         RuntimeVal::Boolean(b) => b,
    //         _ => panic!(),
    //     }
    // }

    fn finish(self) {
        assert_eq!(self.0.len(), 0);
    }
}

fn builtin_print<W: Write>(i: &mut Interpreter<'_, W>, args: Args) -> RuntimeVal {
    // TODO: How to do format strings here, at some point

    if args.0.is_empty() {
        writeln!(i.output).expect("Failed to write to output");
    } else {
        for val in args.0 {
            write!(i.output, "{val}").expect("Failed to write to output"); // TODO: prints a function value
        }
        writeln!(i.output).expect("Failed to write to output");
    }
    RuntimeVal::Null
}

#[allow(clippy::needless_pass_by_value)]
fn builtin_substr(mut args: Args) -> RuntimeVal {
    let string = args.next_string();
    let start = args.next_int();
    let end = args.next_int();
    args.finish();

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

    RuntimeVal::String(string[(start as usize)..(end as usize)].to_string())
}

fn builtin_len(mut args: Args) -> RuntimeVal {
    let string = args.next_string();
    args.finish();

    RuntimeVal::Number(string.len() as f64)
}

fn builtin_clock<W: Write>(i: &mut Interpreter<'_, W>, args: Args) -> RuntimeVal {
    args.finish();

    RuntimeVal::Number((Utc::now() - i.start_time).as_seconds_f64())
}

fn builtin_unix_time(args: Args) -> RuntimeVal {
    args.finish();

    RuntimeVal::Number((Utc::now() - DateTime::UNIX_EPOCH).as_seconds_f64())
}

fn builtin_sleep(mut args: Args) -> RuntimeVal {
    let n = args.next_number();
    args.finish();

    thread::sleep(Duration::from_secs_f64(n));
    RuntimeVal::Null
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::io::{sink, stdout};

    use crate::{
        expr,
        loc::{Loc, Span},
        parser::ExprKind,
    };

    use super::*;

    #[test]
    fn literal() {
        let mut out = stdout();

        let expr = expr!(NumLit(5.0));

        let mut interpreter = Interpreter::test(&mut out);
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

        let mut interpreter = Interpreter::test(&mut out);
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

        let mut interpreter = Interpreter::test(&mut out);
        interpreter.interpret(&expr).unwrap();

        assert_eq!(out, b"bar\n");
    }

    #[test]
    fn binary_ops_int() {
        let mut out = sink();
        let mut interpreter = Interpreter::test(&mut out);

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
        let mut interpreter = Interpreter::test(&mut out);

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
        let mut interpreter = Interpreter::test(&mut out);

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
        let mut interpreter = Interpreter::test(&mut out);

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
        let mut interpreter = Interpreter::test(&mut out);

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
        let mut interpreter = Interpreter::test(&mut out);

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

        let mut interpreter = Interpreter::test(&mut out);

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
