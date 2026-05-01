#[macro_export]
macro_rules! one_token {
    (StrLit($s:expr)) => {
        Ok(Lexeme::new(Token::StringLit(Rc::<str>::from($s)), Span::new(Loc::new(1, 1), Loc::new(1, 1))))
    };
    (Ident($s:expr)) => {
        Ok(Lexeme::new(Token::Ident(Rc::<str>::from($s)), Span::new(Loc::new(1, 1), Loc::new(1, 1))))
    };
    ($name:ident$(($($t:tt)*))?) => {
        Ok(Lexeme::new(Token::$name$(($($t)*))?, Span::new(Loc::new(1, 1), Loc::new(1, 1))))
    };
}

#[macro_export]
macro_rules! tokens {
    ($($name:ident$(($($t:tt)*))?),* $(,)?) => {
        {
            [
                $(
                    one_token!($name$(($($t)*))?)
                ),*
            ]
        }
    };
}

#[macro_export]
macro_rules! expr {
    (Null()) => {
        Expr { kind: ExprKind::Literal(RuntimeVal::Null), span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (NumLit($n:expr)) => {
        Expr { kind: ExprKind::Literal(RuntimeVal::Number($n)), span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (BoolLit($n:expr)) => {
        Expr { kind: ExprKind::Literal(RuntimeVal::Boolean($n)), span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (StrLit($s:expr)) => {
        Expr { kind: ExprKind::Literal(RuntimeVal::String(Rc::<str>::from($s))), span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (Ident($s:expr)) => {
        Expr { kind: ExprKind::Ident(Rc::<str>::from($s)), span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (Binary($ln:ident$l:tt, $op:ident, $rn:ident$r:tt)) => {
        Expr { kind: ExprKind::Binary {
            left: Box::new(expr!($ln$l)),
            op: BinaryOp::$op,
            right: Box::new(expr!($rn$r)),
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (Unary($op:ident, $rn:ident$r:tt)) => {
        Expr { kind: ExprKind::Unary {
            op: UnaryOp::$op,
            right: Box::new(expr!($rn$r)),
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (Call($cn:ident$c:tt, [$($an:ident$a:tt),* $(,)?])) => {
        Expr { kind: ExprKind::Call {
            callee: Box::new(expr!($cn$c)),
            args: vec![$(expr!($an$a)),*],
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (Decl($vn:ident$v:tt)) => {
        Expr { kind: ExprKind::Decl {
            value: Box::new(expr!($vn$v)),
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (VarGet($slot:expr, $depth:expr)) => {
        Expr { kind: ExprKind::VarGet {
            depth: $depth,
            slot: $slot,
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (VarSet($vn:ident$v:tt, $depth:expr, $slot:expr)) => {
        Expr { kind: ExprKind::VarSet {
            value: Box::new(expr!($vn$v)),
            depth: $depth,
            slot: $slot,
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (Block{$($ln:ident$l:tt),* $(,)?}) => {
        Expr { kind: ExprKind::Block(vec![$(expr!($ln$l)),*]), span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (If{$cn:ident$c:tt, Block{$($tn:ident$t:tt),* $(,)?}, None}) => {
        Expr { kind: ExprKind::If {
            cond: Box::new(expr!($cn$c)),
            then: vec![$(expr!($tn$t)),*],
            otherwise: None,
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (If{$cn:ident$c:tt, Block{$($tn:ident$t:tt),* $(,)?}, $on:ident$o:tt}) => {
        Expr { kind: ExprKind::If {
            cond: Box::new(expr!($cn$c)),
            then: vec![$(expr!($tn$t)),*],
            otherwise: Some(Box::new(expr!($on$o))),
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
    (While{$cn:ident$c:tt, Block{$($bn:ident$b:tt),* $(,)?}}) => {
        Expr { kind: ExprKind::While {
            cond: Box::new(expr!($cn$c)),
            body: vec![$(expr!($bn$b)),*],
        }, span: Span::new(Loc::new(1, 1), Loc::new(1, 1)) }
    };
}
