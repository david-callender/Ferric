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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Typ {
    Any,
    Number,
    String,
    Null,
    Function { params: Vec<Typ>, ret: Box<Typ> },
    Bool,
}

impl Typ {
    fn can_coerce(&self, into: &Self) -> bool {
        if self == &Typ::Any || into == &Typ::Any {
            return true;
        }
        self == into
    }
}

impl Display for Typ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Typ::String => write!(f, "string"),
            Typ::Number => write!(f, "number"),
            Typ::Any => write!(f, "any"),
            Typ::Null => write!(f, "null"),
            Typ::Function { params: _, ret: _ } => write!(f, "fn"),
            Typ::Bool => write!(f, "bool"),
        }
    }
}

#[derive(Debug, Default)]
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
    fn consume_parameters(&mut self) -> Res<Vec<(Rc<str>, Typ)>> {
        let mut parameters = Vec::new();
        if self.matches(Token::CloseParen)?.is_none() {
            parameters.push((
                self.consume_ident("Function parameters may only be idents")?,
                self.parse_typ()?
                    .expect("function declarations require types"),
            ));
            while self.matches(Token::Comma)?.is_some() {
                parameters.push((
                    self.consume_ident("Function parameters may only be idents")?,
                    self.parse_typ()?
                        .expect("function declarations require types"),
                ));
            }
            self.consume(
                Token::CloseParen,
                "Unclosed function definition parentheses or missing comma.",
            )?;
        }
        Ok(parameters)
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
            let close = self.consume(
                Token::CloseParen,
                "Unclosed function call parentheses or missing comma",
            )?;
            (args_vec, close.span)
        })
    }

    fn type_of_unaryop(&self, kind: UnaryOp, typ_right: &Typ) -> Typ {
        match (kind, typ_right) {
            (_, Typ::Any) => Typ::Any,
            (UnaryOp::Negate | UnaryOp::BitNot, Typ::Number) => Typ::Number,
            (UnaryOp::BoolNot, Typ::Bool) => Typ::Bool,
            (UnaryOp::Negate, _) => panic!("Unsupported type for numeric negation: {typ_right:?}"),
            (UnaryOp::BitNot, _) => panic!("Unsupported type for bitwise not: {typ_right:?}"),
            (UnaryOp::BoolNot, _) => panic!("Unsupported type for boolean negation: {typ_right:?}"),
        }
    }

    fn type_of_binaryop(&self, typ_left: &Typ, kind: BinaryOp, typ_right: &Typ) -> Typ {
        if *typ_left == Typ::Any || *typ_right == Typ::Any {
            return Typ::Any;
        }
        match kind {
            BinaryOp::Add => match (typ_left, typ_right) {
                (Typ::Number, Typ::Number) => Typ::Number,
                (Typ::String, Typ::String) => Typ::String,
                _ => panic!(
                    "Unsupported types for addition: left: {typ_left:?}, right: {typ_right:?}"
                ),
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
                _ => panic!(
                    "Unsupported types for division: left: {typ_left:?}, right: {typ_right:?}"
                ),
            },
            BinaryOp::Modulo => match (typ_left, typ_right) {
                (Typ::Number, Typ::Number) => Typ::Number,
                _ => {
                    panic!("Unsupported types for modulo: left {typ_left:?}, right: {typ_right:?}")
                }
            },
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::GreaterThan
            | BinaryOp::LessThan
            | BinaryOp::GreaterEq
            | BinaryOp::LessEq => match (typ_left, typ_right) {
                (Typ::Number, Typ::Number) | (Typ::String, Typ::String) => Typ::Bool,
                _ => {
                    panic!(
                        "Unsupported types for comparison: left: {typ_left:?}, right: {typ_right:?}"
                    )
                }
            },
        }
    }

    fn type_of_ident(&self, string: &str) -> Typ {
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
            _ => Typ::Any,
        }
    }

    pub fn parse(&mut self) -> Res<Vec<Expr>> {
        let mut exprs = vec![];
        while self.stream.peek().is_some() {
            let exp = self.parse_expr()?.0;
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

    // assumes the leading Token::OpenBracket has already been consumed. Returns
    // the list of expressions and the span of the closing bracket
    fn parse_block(&mut self, frame: EnvStackFrame) -> Res<(Vec<Expr>, Span, Typ)> {
        if let Some(close) = self.matches(Token::CloseBracket)? {
            return Ok((vec![], close.span, Typ::Null));
        }

        // each block creates its own scope, so add a blank scope to the
        // environment stack.
        self.env.push(frame);

        let first = self.parse_expr()?;
        let mut exprs = vec![first.0];
        let mut typ = first.1;
        while self.matches(Token::Semi)?.is_some() {
            if let Some(close) = self.matches(Token::CloseBracket)? {
                exprs.push(Expr {
                    span: close.span,
                    kind: ExprKind::Literal(RuntimeVal::Null),
                });
                self.env.pop().expect("misaligned environment stack");
                return Ok((exprs, close.span, Typ::Null));
            }
            let next = self.parse_expr()?;
            exprs.push(next.0);
            typ = next.1;
        }
        let close = self.consume(
            Token::CloseBracket,
            "Expected '}' after block. Check for a missing semicolon on the previous line",
        )?;
        self.env.pop().expect("misaligned environment stack");
        Ok((exprs, close.span, typ))
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
                rhs_type.can_coerce(&lhs_type),
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

        let operation = matches_many!(self,
            EqEq(_) => BinaryOp::Equal,
            BangEq(_) => BinaryOp::NotEqual,
            Less(_) => BinaryOp::LessThan,
            LessEq(_) => BinaryOp::LessEq,
            Greater(_) => BinaryOp::GreaterThan,
            GreaterEq(_) => BinaryOp::GreaterEq,
            _ => return Ok((left, typ_left)),
        );

        let (right, typ_right) = self.parse_add_subtract()?;
        let typ = self.type_of_binaryop(&typ_left, operation, &typ_right);
        Ok((Expr::binary(left, operation, right), typ))
    }

    fn parse_add_subtract(&mut self) -> Res<(Expr, Typ)> {
        let (left, typ_left) = self.parse_mult_div_mod()?;

        let operation = matches_many!(self,
            Plus(_) => BinaryOp::Add,
            Minus(_) => BinaryOp::Subtract,
            _ => return Ok((left, typ_left)),
        );

        let (right, typ_right) = self.parse_mult_div_mod()?;
        let typ = self.type_of_binaryop(&typ_left, operation, &typ_right);
        Ok((Expr::binary(left, operation, right), typ))
    }

    fn parse_mult_div_mod(&mut self) -> Res<(Expr, Typ)> {
        let (left, typ_left) = self.parse_unary_op()?;

        let operation = matches_many!(self,
            Star(_) => BinaryOp::Multiply,
            Slash(_) => BinaryOp::Divide,
            Percent(_) => BinaryOp::Modulo,
            _ => return Ok((left, typ_left))
        );
        let (right, typ_right) = self.parse_unary_op()?;
        let typ = self.type_of_binaryop(&typ_left, operation, &typ_right);
        Ok((Expr::binary(left, operation, right), typ))
    }

    fn parse_unary_op(&mut self) -> Res<(Expr, Typ)> {
        let (operation, lexeme) = matches_many!(self,
            Minus(l) => (UnaryOp::Negate, l),
            Tilde(l) => (UnaryOp::BitNot, l),
            Bang(l) => (UnaryOp::BoolNot, l),
            _ => return self.parse_func_call()
        );

        let (right, typ_right) = self.parse_func_call()?;
        let typ = self.type_of_unaryop(operation, &typ_right);
        Ok((Expr::unary(operation, lexeme.span, right), typ))
    }

    fn parse_func_call(&mut self) -> Res<(Expr, Typ)> {
        let (mut func_call, mut func_typ) = self.parse_basic()?;

        while self.matches(Token::OpenParen)?.is_some() {
            let Typ::Function { params, ret } = func_typ else {
                panic!("not a function object");
            };
            let (args_list, close) = self.consume_args()?;
            let mut args = vec![];
            assert_eq!(params.len(), args_list.len(), "different argument counts");
            for ((arg, typ), expected_typ) in args_list.into_iter().zip(params) {
                assert!(&typ.can_coerce(&expected_typ));
                args.push(arg);
            }
            func_call = Expr {
                span: func_call.span + close,
                kind: ExprKind::Call {
                    callee: Box::new(func_call),
                    args,
                },
            };
            func_typ = *ret;
        }

        Ok((func_call, func_typ))
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
        let literal = |val: RuntimeVal, span: Span, typ: Typ| {
            let kind = ExprKind::Literal(val);
            let expr = Expr { span, kind };
            (expr, typ)
        };
        let Lexeme { t: token, span } = self.next()?.ok_or(ParserError::UnexpectedGotNone)?;
        let expr = match token {
            Token::OpenParen => self.parse_paren(span)?,
            Token::StringLit(string) => literal(RuntimeVal::String(string), span, Typ::String),
            Token::NumLit(number) => literal(RuntimeVal::Number(number), span, Typ::Number),
            Token::True => literal(RuntimeVal::Boolean(true), span, Typ::Bool),
            Token::False => literal(RuntimeVal::Boolean(false), span, Typ::Bool),
            Token::Ident(identifier) => self.parse_ident(identifier, span),
            Token::Let => self.parse_decl(span)?,
            Token::Fn => self.parse_func_def(span)?,
            Token::If => self.parse_if(span)?,
            Token::OpenBracket => self.parse_naked_block(span)?,
            Token::While => self.parse_while(span)?,
            Token::For => self.parse_for(span)?,
            t => {
                return Err(ParserError::Unexpected {
                    src: self.src.clone(),
                    actual: Lexeme { t, span },
                });
            }
        };

        Ok(expr)
    }

    // parse_paren assumes that the initial OpenParen token has already
    // been consumed.
    fn parse_paren(&mut self, open: Span) -> Res<(Expr, Typ)> {
        let (inner_expr, inner_typ) = self.parse_expr()?;
        let close = self.consume(Token::CloseParen, "unclosed paren block")?;
        Ok((
            Expr {
                kind: inner_expr.kind,
                span: open + close.span,
            },
            inner_typ,
        ))
    }

    fn parse_ident(&self, identifier: Rc<str>, span: Span) -> (Expr, Typ) {
        let (kind, typ) = if let Some((depth, slot)) = self.find_var(&identifier) {
            let ident_typ = self.env[self.env.len() - depth - 1].typs[slot].clone();
            (ExprKind::VarGet { depth, slot }, ident_typ)
        } else {
            let ident_typ = self.type_of_ident(identifier.as_ref());
            (ExprKind::Ident(identifier), ident_typ)
        };
        (Expr { span, kind }, typ)
    }

    fn parse_decl(&mut self, let_span: Span) -> Res<(Expr, Typ)> {
        let name = self.consume_ident("Expected variable name after let")?;
        let typ_annotation = self.parse_typ()?; // rhs
        self.consume(Token::Eq, "Expected '=' after let")?;
        let (init, init_typ) = self.parse_expr()?; // lhs

        let var_typ = if let Some(t) = typ_annotation {
            assert!(
                init_typ.can_coerce(&t),
                "TypeError: Expected {t}, got {init_typ}"
            );
            t
        } else {
            init_typ
        };
        self.env
            .last_mut()
            .expect("no global env")
            .insert(name, var_typ);

        let span = let_span + init.span;

        let kind = ExprKind::Decl {
            value: Box::new(init),
        };
        let expr = Expr { span, kind };
        Ok((expr, Typ::Null))
    }

    fn parse_func_def(&mut self, fn_span: Span) -> Res<(Expr, Typ)> {
        self.consume(
            Token::OpenParen,
            "Function definition requires an opening parentheses.",
        )?;
        let params = self.consume_parameters()?;
        let param_count = params.len();
        let mut frame = EnvStackFrame::new();
        let mut param_types = vec![];
        for (param, typ) in params {
            frame.insert(param, Typ::Any);
            param_types.push(typ);
        }
        self.consume(
            Token::OpenBracket,
            "Function definition requires an opening bracket.",
        )?;
        let (block, close, body_typ) = self.parse_block(frame)?;

        Ok((
            Expr {
                span: fn_span + close,
                kind: ExprKind::Func {
                    param_count,
                    body: block,
                },
            },
            Typ::Function {
                params: param_types,
                ret: Box::new(body_typ),
            },
        ))
    }

    fn parse_if(&mut self, if_span: Span) -> Res<(Expr, Typ)> {
        let (cond, cond_typ) = self.parse_expr()?;
        assert!(
            cond_typ.can_coerce(&Typ::Bool),
            "conditions must be a boolean"
        );
        self.consume(Token::OpenBracket, "Expected '{' after if condition")?;

        let (then, then_end, then_typ) = self.parse_block(EnvStackFrame::new())?;

        let (otherwise, span) = if self.matches(Token::Otherwise)?.is_some() {
            let (otherwise, span) = match self.next()? {
                Some(Lexeme {
                    t: Token::OpenBracket,
                    span: otherwise_open,
                }) => {
                    let (body, close, otherwise_typ) = self.parse_block(EnvStackFrame::new())?;
                    assert!(
                        then_typ.can_coerce(&otherwise_typ),
                        "then and otherwise must be the same type"
                    );
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
                    let (inner, inner_typ) = self.parse_if(if_span)?;
                    assert!(
                        then_typ.can_coerce(&inner_typ),
                        "then and inner if must be the same type"
                    );
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
            assert!(
                &then_typ.can_coerce(&Typ::Null),
                "if without an otherwise must evaluate to null"
            );
            (None, if_span + then_end)
        };

        Ok((
            Expr {
                kind: ExprKind::If {
                    cond: Box::new(cond),
                    then,
                    otherwise,
                },
                span,
            },
            then_typ,
        ))
    }

    fn parse_naked_block(&mut self, open: Span) -> Res<(Expr, Typ)> {
        let (body, close, typ) = self.parse_block(EnvStackFrame::new())?;
        let kind = ExprKind::Block(body);
        let span = open + close;
        Ok((Expr { span, kind }, typ))
    }

    fn parse_while(&mut self, while_span: Span) -> Res<(Expr, Typ)> {
        let (cond, cond_typ) = self.parse_expr()?;
        assert!(
            &cond_typ.can_coerce(&Typ::Bool),
            "conditions must be a boolean"
        );
        self.consume(Token::OpenBracket, "Expected '{' after while")?;
        let (body, close, _) = self.parse_block(EnvStackFrame::new())?;
        Ok((
            Expr {
                kind: ExprKind::While {
                    cond: Box::new(cond),
                    body,
                },
                span: while_span + close,
            },
            Typ::Null,
        ))
    }

    fn parse_for(&mut self, for_span: Span) -> Res<(Expr, Typ)> {
        self.env.push(EnvStackFrame::new());
        let (init, _) = self.parse_expr()?;
        self.consume(Token::Semi, "Expected ';' after for loop init")?;
        let (cond, cond_typ) = self.parse_expr()?;
        assert!(
            &cond_typ.can_coerce(&Typ::Bool),
            "conditions must be a boolean"
        );
        self.consume(Token::Semi, "Expected ';' after for loop condition")?;

        self.env.push(EnvStackFrame::new());
        let (update, _) = self.parse_expr()?;

        let open = self.consume(Token::OpenBracket, "Expected '{' after 'for'")?;

        let (body, close, _) = self.parse_block(EnvStackFrame::new())?;

        let inner = Expr {
            kind: ExprKind::While {
                cond: Box::new(cond),
                body: vec![
                    Expr {
                        kind: ExprKind::Block(body),
                        span: open.span + close,
                    },
                    update,
                ],
            },
            span: for_span + close,
        };
        Ok((
            Expr {
                kind: ExprKind::Block(vec![init, inner]),
                span: for_span + close,
            },
            Typ::Null,
        ))
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
        assert_eq!(parser.parse_expr().unwrap().0, expr!(NumLit(4.0)));
    }

    #[test]
    fn string_literal() {
        let mut parser = Parser::test(tokens!(StrLit("dingus")));
        assert_eq!(parser.parse_expr().unwrap().0, expr!(StrLit("dingus")));
    }

    #[test]
    fn parentheses() {
        let mut parser = Parser::test(tokens!(OpenParen, NumLit(4.0), CloseParen));
        assert_eq!(parser.parse_expr().unwrap().0, expr!(NumLit(4.0)));
    }

    #[test]
    fn add() {
        let mut parser = Parser::test(tokens!(NumLit(4.0), Plus, NumLit(5.0)));
        let target = expr!(Binary(NumLit(4.0), Add, NumLit(5.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn subtract() {
        let mut parser = Parser::test(tokens!(NumLit(4.0), Minus, NumLit(5.0)));
        let target = expr!(Binary(NumLit(4.0), Subtract, NumLit(5.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn multiply() {
        let mut parser = Parser::test(tokens!(NumLit(20.0), Star, NumLit(22.0)));
        let target = expr!(Binary(NumLit(20.0), Multiply, NumLit(22.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn divide() {
        let mut parser = Parser::test(tokens!(NumLit(20.0), Slash, NumLit(22.0)));
        let target = expr!(Binary(NumLit(20.0), Divide, NumLit(22.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn modulo() {
        let mut parser = Parser::test(tokens!(NumLit(10.0), Percent, NumLit(2.0)));
        let target = expr!(Binary(NumLit(10.0), Modulo, NumLit(2.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn simple_funcall() {
        let mut parser = Parser::test(tokens!(Ident("my_func"), OpenParen, CloseParen));
        let target = expr!(Call(Ident("my_func"), []));
        assert_eq!(parser.parse_expr().unwrap().0, target);
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
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn unary_minus() {
        let mut parser = Parser::test(tokens!(Minus, NumLit(6.0)));
        let target = expr!(Unary(Negate, NumLit(6.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn unary_bitnot() {
        let mut parser = Parser::test(tokens!(Tilde, NumLit(6.0)));
        let target = expr!(Unary(BitNot, NumLit(6.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn unary_and_minus() {
        let mut parser = Parser::test(tokens!(Minus, NumLit(6.0), Minus, NumLit(6.0)));
        let target = expr!(Binary(Unary(Negate, NumLit(6.0)), Subtract, NumLit(6.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
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
        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn block() {
        // empty
        let mut parser = Parser::test(tokens!(OpenBracket, CloseBracket, Semi));
        let target = expr!(Block {});
        assert_eq!(parser.parse_expr().unwrap().0, target);

        // one, return last
        let mut parser = Parser::test(tokens!(OpenBracket, NumLit(4.0), CloseBracket, Semi));
        let target = expr!(Block { NumLit(4.0) });
        assert_eq!(parser.parse_expr().unwrap().0, target);

        // one, don't return last
        let mut parser = Parser::test(tokens!(OpenBracket, NumLit(4.0), Semi, CloseBracket, Semi));
        let target = expr!(Block {
            NumLit(4.0),
            Null()
        });
        assert_eq!(parser.parse_expr().unwrap().0, target);

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
        assert_eq!(parser.parse_expr().unwrap().0, target);

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
        assert_eq!(parser.parse_expr().unwrap().0, target);
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

        assert_eq!(parser.parse_expr().unwrap().0, target);

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

        assert_eq!(parser.parse_expr().unwrap().0, target);

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

        assert_eq!(parser.parse_expr().unwrap().0, target);
    }

    #[test]
    fn comparisons() {
        let mut parser = Parser::test(tokens!(NumLit(3.0), EqEq, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), Equal, NumLit(4.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);

        let mut parser = Parser::test(tokens!(NumLit(3.0), Greater, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), GreaterThan, NumLit(4.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);

        let mut parser = Parser::test(tokens!(NumLit(3.0), LessEq, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), LessEq, NumLit(4.0)));
        assert_eq!(parser.parse_expr().unwrap().0, target);
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
        assert_eq!(parser.parse_expr().unwrap().0, target);
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

        assert_eq!(parser.parse_expr().unwrap().0, target);
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
