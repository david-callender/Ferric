use std::{collections::HashMap, fmt::Display, iter::Peekable, rc::Rc};

use thiserror::Error;

use crate::{
    interpreter::RuntimeVal,
    lexer::{Lexeme, LexerError, Token},
    loc::{ProgramSrc, ProgramSrcInner, Span},
    matches_many,
};

#[derive(Debug, Clone, Error)]
pub enum ParserError {
    #[error(transparent)]
    LexerError(#[from] LexerError),
    #[error("{message}\n{}", .actual.span.format(src, &format!("expected {expected} but found {}", actual.t)))]
    ExpectedActualMismatch {
        src: ProgramSrc,
        expected: Token,
        actual: Lexeme,
        message: &'static str,
    },

    #[error("{message}\nExpected {expected} but found EOF")]
    ExpectedGotNone {
        expected: Token,
        message: &'static str,
    },

    #[error("{message}\n{}", .actual.span.format(src, &format!("expected identifier but found {}", actual.t)))]
    ExpectedIdent {
        src: ProgramSrc,
        actual: Lexeme,
        message: &'static str,
    },

    #[error("{message}\nExpected identifier but found EOF")]
    ExpectedIdentGotNone { message: &'static str },

    #[error("Expression is not assignable\n{}", .span.format(src, "this expression isn't an identifier or hasn't been declared"))]
    InvalidVariableName { src: ProgramSrc, span: Span },

    #[error("Unexpected token\n{}", .actual.span.format(src, &format!("this '{}' token was not expected", .actual.t)))]
    Unexpected { src: ProgramSrc, actual: Lexeme },

    #[error("Unexpected token\nExpected a token but found EOF")]
    UnexpectedGotNone,

    #[error("Invalid otherwise expression\n{}", .actual.span.format(src, &format!("expected '{{' or 'if' after 'otherwise' but found {}", .actual.t)))]
    InvalidOtherwise { src: ProgramSrc, actual: Lexeme },

    #[error("Invalid otherwise expression\nExpected '{{' or 'if' after 'otherwise' but found EOF")]
    InvalidOtherwiseGotNothing,
}

// Module error type
type Res<T> = Result<T, ParserError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(RuntimeVal),
    Ident(Rc<str>),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Decl {
        value: Box<Expr>,
    },
    VarGet {
        depth: usize,
        slot: usize,
    },
    VarSet {
        value: Box<Expr>,
        depth: usize,
        slot: usize,
    },
    Block(Vec<Expr>),
    If {
        cond: Box<Expr>,
        then: Vec<Expr>,
        otherwise: Option<Box<Expr>>,
    },
    While {
        cond: Box<Expr>,
        body: Vec<Expr>,
    },
    Func {
        param_count: usize,
        body: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

impl Expr {
    fn binary(left: Expr, operation: BinaryOp, right: Expr) -> Self {
        Expr {
            span: left.span + right.span,
            kind: ExprKind::Binary {
                left: Box::new(left),
                op: operation,
                right: Box::new(right),
            },
        }
    }

    fn unary(operation: UnaryOp, op_span: Span, right: Expr) -> Self {
        Expr {
            span: op_span + right.span,
            kind: ExprKind::Unary {
                op: operation,
                right: Box::new(right),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEq,
    LessEq,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "add"),
            Self::Subtract => write!(f, "subtract"),
            Self::Multiply => write!(f, "multiply"),
            Self::Divide => write!(f, "divide"),
            Self::Modulo => write!(f, "calculate modulo between"),
            Self::Equal
            | Self::NotEqual
            | Self::GreaterThan
            | Self::LessThan
            | Self::GreaterEq
            | Self::LessEq => write!(f, "compare"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Negate,
    BitNot,
    BoolNot,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negate => write!(f, "negate"),
            Self::BitNot => write!(f, "bitwise not"),
            Self::BoolNot => write!(f, "boolean negate"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Typ {
    Any,
    Number,
    String,
    Null,
    Function { params: Vec<Typ>, ret: Box<Typ> },
    Bool,
}

impl Eq for Typ {}

impl Display for Typ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Typ::String => write!(f, "String"),
            Typ::Number => write!(f, "Number"),
            Typ::Any => write!(f, "Any"),
            Typ::Null => write!(f, "Null"),
            Typ::Function { params: _, ret: _ } => write!(f, "Function"),
            Typ::Bool => write!(f, "Unknown"),
        }
    }
}

#[derive(Default)]
pub struct EnvStackFrame {
    next_index: usize,
    frame: HashMap<Rc<str>, usize>,
    typs: Vec<Typ>,
}

pub struct Parser<I: Iterator<Item = Result<Lexeme, LexerError>>> {
    stream: Peekable<I>,
    src: ProgramSrc,
    env: Vec<EnvStackFrame>,
}

impl EnvStackFrame {
    pub fn new() -> Self {
        EnvStackFrame::default()
    }
    pub fn insert(&mut self, name: Rc<str>, typ: Typ) {
        self.frame.insert(name, self.next_index);
        self.typs.push(typ);
        self.next_index += 1;
    }
    pub fn get(&self, key: &str) -> Option<usize> {
        Some(*self.frame.get(key)?)
    }
}

fn type_of_unaryop(kind: UnaryOp, typ_right: &Typ) -> Typ {
    match (kind, typ_right) {
        (_, Typ::Any) => Typ::Any,
        (UnaryOp::Negate | UnaryOp::BitNot, Typ::Number) => Typ::Number,
        (UnaryOp::BoolNot, Typ::Bool) => Typ::Bool,
        (UnaryOp::Negate, _) => panic!("Unsupported type for numeric negation: {typ_right:?}"),
        (UnaryOp::BitNot, _) => panic!("Unsupported type for bitwise not: {typ_right:?}"),
        (UnaryOp::BoolNot, _) => panic!("Unsupported type for boolean negation: {typ_right:?}"),
    }
}

fn type_of_binaryop(typ_left: &Typ, kind: BinaryOp, typ_right: &Typ) -> Typ {
    if *typ_left == Typ::Any || *typ_right == Typ::Any {
        return Typ::Any;
    }
    match kind {
        BinaryOp::Add => match (typ_left, typ_right) {
            (Typ::Number, Typ::Number) => Typ::Number,
            (Typ::String, Typ::String) => Typ::String,
            _ => panic!("Unsupported types for addition: left: {typ_left:?}, right: {typ_right:?}"),
        },
        BinaryOp::Subtract => match (typ_left, typ_right) {
            (Typ::Number, Typ::Number) => Typ::Number,
            _ => panic!(
                "Unsupported types for subtraction: left: {typ_left:?}, right: {typ_right:?}"
            ),
        },
        BinaryOp::Multiply => match (typ_left, typ_right) {
            (Typ::Number, Typ::Number) => Typ::Number,
            (Typ::String, Typ::Number) => Typ::String,
            _ => panic!(
                "Unsupported types for multiplication: left {typ_left:?}, right: {typ_right:?}"
            ),
        },
        BinaryOp::Divide => match (typ_left, typ_right) {
            (Typ::Number, Typ::Number) => Typ::Number,
            _ => panic!("Unsupported types for division: left: {typ_left:?}, right: {typ_right:?}"),
        },
        BinaryOp::Modulo => match (typ_left, typ_right) {
            (Typ::Number, Typ::Number) => Typ::Number,
            _ => panic!("Unsupported types for modulo: left {typ_left:?}, right: {typ_right:?}"),
        },
        BinaryOp::Equal
        | BinaryOp::NotEqual
        | BinaryOp::GreaterThan
        | BinaryOp::LessThan
        | BinaryOp::GreaterEq
        | BinaryOp::LessEq => match (typ_left, typ_right) {
            (Typ::Number, Typ::Number) | (Typ::String, Typ::String) => Typ::Bool,
            _ => {
                panic!("Unsupported types for comparison: left: {typ_left:?}, right: {typ_right:?}")
            }
        },
    }
}

fn type_of_ident(string: &str) -> Typ {
    match string {
        "print" => Typ::Function {
            params: vec![Typ::Any],
            ret: Box::new(Typ::Null),
        },
        "substr" => Typ::Function {
            params: vec![Typ::String, Typ::Number, Typ::Number],
            ret: Box::new(Typ::String),
        },
        "len" => Typ::Function {
            params: vec![Typ::String],
            ret: Box::new(Typ::Number),
        },
        "clock" | "unix_time" => Typ::Function {
            params: vec![],
            ret: Box::new(Typ::Number),
        },
        _ => panic!("Got ident which does not resolve to function: {string}"),
    }
}

impl<I: Iterator<Item = Result<Lexeme, LexerError>>> Parser<I> {
    pub fn new<II: IntoIterator<IntoIter = I>>(stream: II, src: ProgramSrc) -> Self {
        Self {
            stream: stream.into_iter().peekable(),
            src,
            env: vec![EnvStackFrame::new()], // global
        }
    }

    pub fn test<II: IntoIterator<IntoIter = I>>(stream: II) -> Self {
        Self {
            stream: stream.into_iter().peekable(),
            src: Rc::new(ProgramSrcInner::new(String::new())),
            env: vec![EnvStackFrame::new()], // global
        }
    }

    fn next(&mut self) -> Res<Option<Lexeme>> {
        Ok(self.stream.next().transpose()?)
    }

    fn peek(&mut self) -> Res<Option<&Lexeme>> {
        let peeked = self.stream.peek();
        match peeked {
            Some(Ok(lexeme)) => Ok(Some(lexeme)),
            Some(Err(err)) => Err(ParserError::LexerError(err.clone())),
            None => Ok(None),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn matches(&mut self, expected: Token) -> Res<Option<Lexeme>> {
        Ok(if self.peek()?.is_some_and(|x| x.t == expected) {
            Some(self.next()?.unwrap())
        } else {
            None
        })
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_one_of<const N: usize>(&mut self, expected: [Token; N]) -> Option<Token> {
        if self
            .stream
            .peek()
            .is_some_and(|x| x.as_ref().is_ok_and(|x| expected.contains(&x.t)))
        {
            return Some(self.stream.next().unwrap().unwrap().t);
        }
        None
    }

    fn consume(&mut self, expected: Token, message: &'static str) -> Res<Lexeme> {
        let token = self.next()?.ok_or_else(|| ParserError::ExpectedGotNone {
            expected: expected.clone(),
            message,
        })?;
        if token.t == expected {
            Ok(token)
        } else {
            Err(ParserError::ExpectedActualMismatch {
                src: self.src.clone(),
                expected,
                actual: token,
                message,
            })
        }
    }

    fn consume_ident(&mut self, message: &'static str) -> Res<Rc<str>> {
        match self.next()? {
            Some(Lexeme {
                t: Token::Ident(name),
                span: _,
            }) => Ok(name),
            Some(actual) => Err(ParserError::ExpectedIdent {
                src: self.src.clone(),
                actual,
                message,
            }),
            _ => Err(ParserError::ExpectedIdentGotNone { message }),
        }
    }

    // consume_parameters assumes that the initial opening paren has
    // already been parsed.
    fn consume_parameters(&mut self) -> Res<Vec<Rc<str>>> {
        let mut parameters: Vec<Rc<str>> = Vec::new();
        if self.matches(Token::CloseParen)?.is_none() {
            parameters.push(self.consume_ident("Function parameters may only be idents")?);
            while self.matches(Token::Comma)?.is_some() {
                parameters.push(self.consume_ident("Function parameters may only be idents")?);
            }
            self.consume(
                Token::CloseParen,
                "Unclosed function definition parentheses or missing comma.",
            )?;
        }
        Ok(parameters)
    }

    fn type_of_block(&self, exprs: &[Expr]) -> Typ {
        let mut typ = Typ::Null;
        for expr in exprs {
            typ = self.type_of(expr);
        }
        typ
    }

    fn type_of(&self, expr: &Expr) -> Typ {
        match &expr.kind {
            ExprKind::VarSet { .. } | ExprKind::Decl { .. } => Typ::Null,
            ExprKind::While { cond, body } => {
                assert_eq!(
                    self.type_of(cond),
                    Typ::Bool,
                    "While conditions must be of boolean type",
                );
                self.type_of_block(body);
                Typ::Null
            }
            ExprKind::Literal(kind) => match kind {
                RuntimeVal::Number(_) => Typ::Number,
                RuntimeVal::String(_) => Typ::String,
                RuntimeVal::Boolean(_) => Typ::Bool,
                RuntimeVal::Null => Typ::Null,
                RuntimeVal::Function(_) => unreachable!("Function found in literal expr"),
            },
            ExprKind::VarGet { depth, slot } => {
                assert!(
                    *depth < self.env.len(),
                    "Variable get references greater depth than stack size"
                );
                assert!(
                    *slot < self.env[*depth].typs.len(),
                    "Variable get references greater slot number than exists in referenced stack frame"
                );
                self.env[*depth].typs[*slot].clone()
            }
            ExprKind::Unary { op, right } => type_of_unaryop(*op, &self.type_of(right.as_ref())),
            ExprKind::Binary { left, op, right } => type_of_binaryop(
                &self.type_of(left.as_ref()),
                *op,
                &self.type_of(right.as_ref()),
            ),
            ExprKind::Call { callee, args: _ } => match self.type_of(callee) {
                Typ::Any => Typ::Any,
                Typ::Function { params: _, ret } => *ret,
                _ => panic!("Attempted to call non-function expr"),
            },
            ExprKind::Block(exprs) => {
                for expr in &exprs[..exprs.len() - 1] {
                    self.type_of(expr);
                }
                self.type_of(exprs.last().expect("Block is entirely empty"))
            }
            ExprKind::Func {
                param_count: _,
                body,
            } => Typ::Function {
                params: Vec::new(),
                ret: Box::new(self.type_of(body.last().expect("Function body is entirely empty"))),
            },
            ExprKind::If {
                cond,
                then,
                otherwise,
            } => {
                assert_eq!(
                    Typ::Bool,
                    self.type_of(cond.as_ref()),
                    "If condition is not of boolean type"
                );
                let typ_then = self.type_of_block(then);
                if let Some(other) = otherwise {
                    let typ_otherwise = self.type_of(other.as_ref());
                    assert_eq!(
                        typ_then, typ_otherwise,
                        "All arms of an if-otherwise chain must evaluate to the same type"
                    );
                }
                typ_then
            }
            ExprKind::Ident(string) => type_of_ident(string.as_ref()),
        }
    }

    pub fn parse(&mut self) -> Res<Vec<Expr>> {
        let mut exprs = vec![];
        while self.stream.peek().is_some() {
            let exp = self.parse_expr()?.0;
            self.type_of(&exp); // panics if type error
            exprs.push(exp);
            self.consume(Token::Semi, "Expected ';' after expression")?;
        }
        Ok(exprs)
    }

    fn parse_typ(&mut self) -> Res<Option<Typ>> {
        if self.matches(Token::Colon)?.is_some() {
            let typ = match self.next()?.map(|l| l.t) {
                Some(Token::Null) => Typ::Null,
                Some(Token::Number) => Typ::Number,
                Some(Token::String) => Typ::String,
                Some(Token::Bool) => Typ::Bool,
                Some(Token::Any) => Typ::Any,
                Some(x) => panic!("Expected type, got {x}"),
                None => panic!("Expected type"),
            };
            Ok(Some(typ))
        } else {
            Ok(None)
        }
    }

    fn parse_expr(&mut self) -> Res<(Expr, Typ)> {
        self.parse_var_set()
    }

    fn parse_var_set(&mut self) -> Res<(Expr, Typ)> {
        let (left, typ_left) = self.parse_comparisons()?;

        if self.matches(Token::Eq)?.is_some() {
            let (right, typ_right) = self.parse_comparisons()?;

            let ExprKind::VarGet { slot, depth } = left.kind else {
                return Err(ParserError::InvalidVariableName {
                    src: self.src.clone(),
                    span: left.span,
                });
            };

            let span = left.span + right.span;

            let rhs_type = typ_right;
            let lhs_type = self.env[depth].typs[slot].clone();

            assert!(
                rhs_type == lhs_type,
                "Type error: expected {rhs_type}, got {lhs_type}"
            );

            return Ok((
                Expr {
                    kind: ExprKind::VarSet {
                        value: Box::new(right),
                        depth,
                        slot,
                    },
                    span,
                },
                Typ::Null,
            ));
        }
        Ok((left, typ_left))
    }

    fn parse_comparisons(&mut self) -> Res<(Expr, Typ)> {
        let (left, typ_left) = self.parse_add_subtract()?;

        let operation = match self.is_one_of([
            Token::Greater,
            Token::GreaterEq,
            Token::Less,
            Token::LessEq,
            Token::EqEq,
            Token::BangEq,
        ]) {
            Some(Token::EqEq) => BinaryOp::Equal,
            Some(Token::BangEq) => BinaryOp::NotEqual,
            Some(Token::Less) => BinaryOp::LessThan,
            Some(Token::LessEq) => BinaryOp::LessEq,
            Some(Token::Greater) => BinaryOp::GreaterThan,
            Some(Token::GreaterEq) => BinaryOp::GreaterEq,
            Some(_) => unreachable!(),
            None => return Ok((left, typ_left)),
        };
        let (right, typ_right) = self.parse_add_subtract()?;
        let typ = match (typ_left, typ_right) {
            (Typ::Number, Typ::Number) | (Typ::String, Typ::String) => Typ::Bool,
            (typ_left, typ_right) => {
                panic!("Unsupported types for comparison: left: {typ_left:?}, right: {typ_right:?}")
            }
        };
        Ok((Expr::binary(left, operation, right), typ))
    }

    fn parse_add_subtract(&mut self) -> Res<(Expr, Typ)> {
        let (left, typ_left) = self.parse_mult_div_mod()?;

        if self.matches(Token::Plus)?.is_some() {
            let (right, typ_right) = self.parse_mult_div_mod()?;
            let typ = match (typ_left, typ_right) {
                (Typ::Number, Typ::Number) => Typ::Number,
                (Typ::String, Typ::String) => Typ::String,
                (typ_left, typ_right) => panic!(
                    "Unsupported types for addition: left: {typ_left:?}, right: {typ_right:?}"
                ),
            };
            return Ok((Expr::binary(left, BinaryOp::Add, right), typ));
        } else if self.matches(Token::Minus)?.is_some() {
            let (right, typ_right) = self.parse_mult_div_mod()?;
            let typ = match (typ_left, typ_right) {
                (Typ::Number, Typ::Number) => Typ::Number,
                (typ_left, typ_right) => panic!(
                    "Unsupported types for subtraction: left: {typ_left:?}, right: {typ_right:?}"
                ),
            };
            return Ok((Expr::binary(left, BinaryOp::Subtract, right), typ));
        }
        Ok((left, typ_left))
    }

    fn parse_mult_div_mod(&mut self) -> Res<(Expr, Typ)> {
        let left = self.parse_unary_op()?;
        let operation = match self.is_one_of([Token::Star, Token::Slash, Token::Percent]) {
            Some(Token::Star) => BinaryOp::Multiply,
            Some(Token::Slash) => BinaryOp::Divide,
            Some(Token::Percent) => BinaryOp::Modulo,
            Some(_) => unreachable!(),
            None => return Ok(left),
        };
        let right = self.parse_unary_op()?;
        Ok(Expr::binary(left, operation, right))
    }

    fn parse_unary_op(&mut self) -> Res<(Expr, Typ)> {
        if let Some(lexeme) = self.matches(Token::Minus)? {
            let right = self.parse_func_call()?;
            return Ok(Expr::unary(UnaryOp::Negate, lexeme.span, right));
        } else if let Some(lexeme) = self.matches(Token::Tilde)? {
            let right = self.parse_func_call()?;
            return Ok(Expr::unary(UnaryOp::BitNot, lexeme.span, right));
        } else if let Some(lexeme) = self.matches(Token::Bang)? {
            let right = self.parse_func_call()?;
            return Ok(Expr::unary(UnaryOp::BoolNot, lexeme.span, right));
        }
        self.parse_func_call()
    }

    // consume_args also consumes the closing paren of the
    // arguments list, but assumes that the opening paren has
    // already been parsed. Also returns the closing param's span
    fn consume_args(&mut self) -> Res<(Vec<(Expr, Typ)>, Span)> {
        Ok(if let Some(close) = self.matches(Token::CloseParen)? {
            (vec![], close.span)
        } else {
            let mut args_vec = vec![];
            args_vec.push(self.parse_expr()?);
            while self.matches(Token::Comma)?.is_some() {
                args_vec.push(self.parse_expr()?);
            }
            (
                args_vec,
                self.consume(
                    Token::CloseParen,
                    "Unclosed function call parentheses or missing comma",
                )?
                .span,
            )
        })
    }

    fn parse_func_call(&mut self) -> Res<(Expr, Typ)> {
        let mut func_call = self.parse_basic()?;

        while self.matches(Token::OpenParen)?.is_some() {
            let (args_list, close) = self.consume_args()?;
            func_call = Expr {
                span: func_call.span + close,
                kind: ExprKind::Call {
                    callee: Box::new(func_call),
                    args: args_list,
                },
            };
        }

        Ok(func_call)
    }

    fn find_var(&self, name: &str) -> Option<(usize, usize)> {
        for (depth, stack_frame) in self.env.iter().rev().enumerate() {
            if let Some(index) = stack_frame.get(name) {
                return Some((depth, index));
            }
        }
        None
    }

    fn parse_basic(&mut self) -> Res<(Expr, Typ)> {
        let lexeme = self.next()?.ok_or(ParserError::UnexpectedGotNone)?;
        let expr = match lexeme.t {
            Token::OpenParen => self.parse_paren(lexeme.span)?,
            Token::StringLit(string) => Expr {
                span: lexeme.span,
                kind: ExprKind::Literal(RuntimeVal::String(string)),
            },
            Token::NumLit(number) => Expr {
                span: lexeme.span,
                kind: ExprKind::Literal(RuntimeVal::Number(number)),
            },
            Token::True => Expr {
                span: lexeme.span,
                kind: ExprKind::Literal(RuntimeVal::Boolean(true)),
            },
            Token::False => Expr {
                span: lexeme.span,
                kind: ExprKind::Literal(RuntimeVal::Boolean(false)),
            },
            Token::Ident(identifier) => self
                .find_var(&identifier)
                .map(|(depth, slot)| Expr {
                    span: lexeme.span,
                    kind: ExprKind::VarGet { depth, slot },
                })
                .unwrap_or(Expr {
                    span: lexeme.span,
                    kind: ExprKind::Ident(identifier),
                }),
            Token::Let => self.parse_decl(lexeme.span)?,
            Token::Fn => self.parse_func_def(lexeme.span)?,
            Token::If => self.parse_if(lexeme.span)?,
            Token::OpenBracket => {
                let (body, close) = self.parse_block(EnvStackFrame::new())?;
                Expr {
                    kind: ExprKind::Block(body),
                    span: lexeme.span + close,
                }
            }
            Token::While => self.parse_while(lexeme.span)?,
            _ => {
                return Err(ParserError::Unexpected {
                    src: self.src.clone(),
                    actual: lexeme,
                });
            }
        };

        Ok(expr)
    }

    // parse_paren assumes that the initial OpenParen token has already
    // been consumed.
    fn parse_paren(&mut self, open: Span) -> Res<(Expr, Typ)> {
        let inner_expr = self.parse_expr()?;
        let close = self.consume(Token::CloseParen, "unclosed paren block")?;
        Ok(Expr {
            kind: inner_expr.kind,
            span: open + close.span,
        })
    }

    fn parse_decl(&mut self, let_span: Span) -> Res<(Expr, Typ)> {
        let name = self.consume_ident("Expected variable name after let")?;
        let typ = self.parse_typ()?; // rhs
        self.consume(Token::Eq, "Expected '=' after let")?;
        let init = self.parse_expr()?; // lhs

        let lhs_type = self.type_of(&init);

        if let Some(t) = typ {
            assert!(lhs_type == t, "TypeError: Expected {lhs_type}, got {t}");

            self.env.last_mut().expect("no global env").insert(name, t);
        } else {
            self.env
                .last_mut()
                .expect("no global env")
                .insert(name, lhs_type);
        }

        let span = let_span + init.span;

        let kind = ExprKind::Decl {
            value: Box::new(init),
        };
        let expr = Expr { span, kind };
        Ok(expr)
    }

    fn parse_func_def(&mut self, fn_span: Span) -> Res<(Expr, Typ)> {
        self.consume(
            Token::OpenParen,
            "Function definition requires an opening parentheses.",
        )?;
        let params = self.consume_parameters()?;
        let param_count = params.len();
        let mut frame = EnvStackFrame::new();
        for param in params {
            frame.insert(param, Typ::Any);
        }
        self.consume(
            Token::OpenBracket,
            "Function definition requires an opening bracket.",
        )?;
        let (block, close) = self.parse_block(frame)?;

        Ok(Expr {
            span: fn_span + close,
            kind: ExprKind::Func {
                param_count,
                body: block,
            },
        })
    }

    fn parse_if(&mut self, if_span: Span) -> Res<(Expr, Typ)> {
        let cond = self.parse_expr()?;
        self.consume(Token::OpenBracket, "Expected '{' after if condition")?;

        let (then, then_end) = self.parse_block(EnvStackFrame::new())?;

        let (otherwise, span) = if self.matches(Token::Otherwise)?.is_some() {
            let (otherwise, span) = match self.next()? {
                Some(Lexeme {
                    t: Token::OpenBracket,
                    span: otherwise_open,
                }) => {
                    let (body, close) = self.parse_block(EnvStackFrame::new())?;
                    (
                        (Expr {
                            kind: ExprKind::Block(body),
                            span: otherwise_open + close,
                        }),
                        if_span + close,
                    )
                }
                Some(Lexeme {
                    t: Token::If,
                    span: if_span,
                }) => {
                    let inner = self.parse_if(if_span)?;
                    let span = inner.span;
                    ((inner), span)
                }
                Some(other) => {
                    return Err(ParserError::InvalidOtherwise {
                        src: self.src.clone(),
                        actual: other,
                    });
                }
                None => return Err(ParserError::InvalidOtherwiseGotNothing),
            };

            (Some(Box::new(otherwise)), span)
        } else {
            (None, if_span + then_end)
        };

        Ok(Expr {
            kind: ExprKind::If {
                cond: Box::new(cond),
                then,
                otherwise,
            },
            span,
        })
    }

    fn parse_while(&mut self, while_span: Span) -> Res<(Expr, Typ)> {
        let cond = Box::new(self.parse_expr()?);
        self.consume(Token::OpenBracket, "Expected '{' after while")?;
        let (body, close) = self.parse_block(EnvStackFrame::new())?;
        Ok(Expr {
            kind: ExprKind::While { cond, body },
            span: while_span + close,
        })
    }

    // assumes the leading Token::OpenBracket has already been consumed. Returns
    // the list of expressions and the span of the closing bracket
    fn parse_block(&mut self, frame: EnvStackFrame) -> Res<(Vec<Expr>, Span)> {
        if let Some(close) = self.matches(Token::CloseBracket)? {
            return Ok((vec![], close.span));
        }

        // each block creates its own scope, so add a blank scope to the
        // environment stack.
        self.env.push(frame);

        let mut exprs = vec![self.parse_expr()?];
        while self.matches(Token::Semi)?.is_some() {
            if let Some(close) = self.matches(Token::CloseBracket)? {
                exprs.push(Expr {
                    span: close.span,
                    kind: ExprKind::Literal(RuntimeVal::Null),
                });
                self.env.pop().expect("misaligned environment stack");
                return Ok((exprs, close.span));
            }
            exprs.push(self.parse_expr()?);
        }
        let close = self.consume(
            Token::CloseBracket,
            "Expected '}' after block. Check for a missing semicolon on the previous line",
        )?;
        self.env.pop().expect("misaligned environment stack");
        Ok((exprs, close.span))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expr,
        loc::{Loc, Span},
        one_token, tokens,
    };

    use super::*;

    #[test]
    fn num_literal() {
        let mut parser = Parser::test(tokens!(NumLit(4.0)));
        assert_eq!(parser.parse_expr().unwrap(), expr!(NumLit(4.0)));
    }

    #[test]
    fn string_literal() {
        let mut parser = Parser::test(tokens!(StrLit("dingus")));
        assert_eq!(parser.parse_expr().unwrap(), expr!(StrLit("dingus")));
    }

    #[test]
    fn parentheses() {
        let mut parser = Parser::test(tokens!(OpenParen, NumLit(4.0), CloseParen));
        assert_eq!(parser.parse_expr().unwrap(), expr!(NumLit(4.0)));
    }

    #[test]
    fn add() {
        let mut parser = Parser::test(tokens!(NumLit(4.0), Plus, NumLit(5.0)));
        let target = expr!(Binary(NumLit(4.0), Add, NumLit(5.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn subtract() {
        let mut parser = Parser::test(tokens!(NumLit(4.0), Minus, NumLit(5.0)));
        let target = expr!(Binary(NumLit(4.0), Subtract, NumLit(5.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn multiply() {
        let mut parser = Parser::test(tokens!(NumLit(20.0), Star, NumLit(22.0)));
        let target = expr!(Binary(NumLit(20.0), Multiply, NumLit(22.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn divide() {
        let mut parser = Parser::test(tokens!(NumLit(20.0), Slash, NumLit(22.0)));
        let target = expr!(Binary(NumLit(20.0), Divide, NumLit(22.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn modulo() {
        let mut parser = Parser::test(tokens!(NumLit(10.0), Percent, NumLit(2.0)));
        let target = expr!(Binary(NumLit(10.0), Modulo, NumLit(2.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn simple_funcall() {
        let mut parser = Parser::test(tokens!(Ident("my_func"), OpenParen, CloseParen));
        let target = expr!(Call(Ident("my_func"), []));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn complex_funcall() {
        let mut parser = Parser::test(tokens!(
            Ident("my_func"),
            OpenParen,
            NumLit(42.0),
            Comma,
            NumLit(88.0),
            CloseParen,
            OpenParen,
            StrLit("dingus"),
            CloseParen
        ));
        let target = expr!(Call(
            Call(Ident("my_func"), [NumLit(42.0), NumLit(88.0)]),
            [StrLit("dingus")]
        ));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn unary_minus() {
        let mut parser = Parser::test(tokens!(Minus, NumLit(6.0)));
        let target = expr!(Unary(Negate, NumLit(6.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn unary_bitnot() {
        let mut parser = Parser::test(tokens!(Tilde, NumLit(6.0)));
        let target = expr!(Unary(BitNot, NumLit(6.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn unary_and_minus() {
        let mut parser = Parser::test(tokens!(Minus, NumLit(6.0), Minus, NumLit(6.0)));
        let target = expr!(Binary(Unary(Negate, NumLit(6.0)), Subtract, NumLit(6.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn unary_with_parens() {
        let mut parser = Parser::test(tokens!(
            Minus,
            OpenParen,
            NumLit(6.0),
            Plus,
            NumLit(6.0),
            CloseParen
        ));
        let target = expr!(Unary(Negate, Binary(NumLit(6.0), Add, NumLit(6.0))));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn block() {
        // empty
        let mut parser = Parser::test(tokens!(OpenBracket, CloseBracket, Semi));
        let target = expr!(Block {});
        assert_eq!(parser.parse_expr().unwrap(), target);

        // one, return last
        let mut parser = Parser::test(tokens!(OpenBracket, NumLit(4.0), CloseBracket, Semi));
        let target = expr!(Block { NumLit(4.0) });
        assert_eq!(parser.parse_expr().unwrap(), target);

        // one, don't return last
        let mut parser = Parser::test(tokens!(OpenBracket, NumLit(4.0), Semi, CloseBracket, Semi));
        let target = expr!(Block {
            NumLit(4.0),
            Null()
        });
        assert_eq!(parser.parse_expr().unwrap(), target);

        // many, return last
        let mut parser = Parser::test(tokens!(
            OpenBracket,
            NumLit(4.0),
            Semi,
            NumLit(5.0),
            CloseBracket,
            Semi
        ));
        let target = expr!(Block {
            NumLit(4.0),
            NumLit(5.0)
        });
        assert_eq!(parser.parse_expr().unwrap(), target);

        // many, don't return last
        let mut parser = Parser::test(tokens!(
            OpenBracket,
            NumLit(4.0),
            Semi,
            NumLit(5.0),
            Semi,
            CloseBracket,
            Semi
        ));
        let target = expr!(Block {
            NumLit(4.0),
            NumLit(5.0),
            Null()
        });
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn if_expr() {
        // if
        let mut parser = Parser::test(tokens!(
            If,
            NumLit(1.0),
            OpenBracket,
            NumLit(4.0),
            CloseBracket,
            Semi
        ));

        let target = expr!(If {
            NumLit(1.0),
            Block { NumLit(4.0) },
            None
        });

        assert_eq!(parser.parse_expr().unwrap(), target);

        // if otherwise
        let mut parser = Parser::test(tokens!(
            If,
            NumLit(1.0),
            OpenBracket,
            NumLit(4.0),
            CloseBracket,
            Otherwise,
            OpenBracket,
            NumLit(5.0),
            CloseBracket,
            Semi
        ));

        let target = expr!(If {
            NumLit(1.0),
            Block { NumLit(4.0) },
            Block { NumLit(5.0) }
        });

        assert_eq!(parser.parse_expr().unwrap(), target);

        // if otherwise-if otherwise
        let mut parser = Parser::test(tokens!(
            If,
            NumLit(1.0),
            OpenBracket,
            NumLit(4.0),
            CloseBracket,
            Otherwise,
            If,
            NumLit(2.0),
            OpenBracket,
            NumLit(5.0),
            CloseBracket,
            Otherwise,
            OpenBracket,
            NumLit(6.0),
            CloseBracket,
            Semi
        ));

        let target = expr!(If {
            NumLit(1.0),
            Block { NumLit(4.0) },
            If {
                NumLit(2.0),
                Block { NumLit(5.0) },
                Block { NumLit(6.0) }
            }
        });

        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn comparisons() {
        let mut parser = Parser::test(tokens!(NumLit(3.0), EqEq, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), Equal, NumLit(4.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);

        let mut parser = Parser::test(tokens!(NumLit(3.0), Greater, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), GreaterThan, NumLit(4.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);

        let mut parser = Parser::test(tokens!(NumLit(3.0), LessEq, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), LessEq, NumLit(4.0)));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn comparison_order() {
        let mut parser = Parser::test(tokens!(
            NumLit(3.0),
            Plus,
            NumLit(3.0),
            EqEq,
            NumLit(9.0),
            Minus,
            NumLit(3.0)
        ));
        let target = expr!(Binary(
            Binary(NumLit(3.0), Add, NumLit(3.0)),
            Equal,
            Binary(NumLit(9.0), Subtract, NumLit(3.0))
        ));
        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    #[test]
    fn while_expr() {
        let mut parser = Parser::test(tokens!(
            While,
            NumLit(1.0),
            Greater,
            NumLit(5.0),
            OpenBracket,
            NumLit(6.9),
            CloseBracket,
        ));
        let target = expr!(While {
            Binary (NumLit(1.0), GreaterThan, NumLit(5.0)),
            Block {
                NumLit(6.9),
            }
        });

        assert_eq!(parser.parse_expr().unwrap(), target);
    }

    // #[test]
    // pub fn function_definition() {
    //     let mut parser = Parser::test(tokens![
    //         Fn,
    //         OpenParen,
    //         Ident("param1".to_string()),
    //         Comma,
    //         Ident("Param2".to_string()),
    //         CloseParen,
    //         OpenBracket,
    //         NumLit(42.0),
    //         CloseBracket,
    //     ]);
    //     let target = expr!(Func {
    //         [Ident("param1"), Ident("param2")],
    //         Block {
    //     	NumLit(42.0),
    //         }
    //     });
    //     assert_eq!(parser.parse(), target);
    // }
}
