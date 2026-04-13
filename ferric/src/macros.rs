#[macro_export]
macro_rules! one_token {
    (StrLit($s:expr)) => {
        Ok(Lexeme::new(Token::StringLit(String::from($s)), Span::new(Loc::new(1, 1), Loc::new(1, 1))))
    };
    (Ident($s:expr)) => {
        Ok(Lexeme::new(Token::Ident(String::from($s)), Span::new(Loc::new(1, 1), Loc::new(1, 1))))
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
        Expr::Literal(RuntimeVal::Null)
    };
    (NumLit($n:expr)) => {
        Expr::Literal(RuntimeVal::Number($n))
    };
    (BoolLit($n:expr)) => {
        Expr::Literal(RuntimeVal::Boolean($n))
    };
    (StrLit($s:expr)) => {
        Expr::Literal(RuntimeVal::String(String::from($s)))
    };
    (Ident($s:expr)) => {
        Expr::Ident(String::from($s))
    };
    (Binary($ln:ident$l:tt, $op:ident, $rn:ident$r:tt)) => {
        Expr::Binary {
            left: Box::new(expr!($ln$l)),
            operation: BinaryOp::$op,
            right: Box::new(expr!($rn$r)),
        }
    };
    (Unary($op:ident, $rn:ident$r:tt)) => {
        Expr::Unary {
            operation: UnaryOp::$op,
            right: Box::new(expr!($rn$r)),
        }
    };
    (Call($cn:ident$c:tt, [$($an:ident$a:tt),* $(,)?])) => {
        Expr::Call {
            callee: Box::new(expr!($cn$c)),
            args: vec![$(expr!($an$a)),*],
        }
    };
    (Decl($vn:ident$v:tt, $slot:expr)) => {
        Expr::Decl {
            value: Box::new(expr!($vn$v)),
            slot: $slot,
        }
    };
    (VarGet($slot:expr)) => {
        Expr::VarGet {
            slot: $slot,
        }
    };
    (VarSet($vn:ident$v:tt, $slot:expr)) => {
        Expr::VarSet {
            value: Box::new(expr!($vn$v)),
            slot: $slot,
        }
    };
    (Block{$($ln:ident$l:tt),* $(,)?}) => {
        Expr::Block(vec![$(expr!($ln$l)),*])
    };
    (If{$cn:ident$c:tt, $tn:ident$t:tt, None}) => {
        Expr::If {
            cond: Box::new(expr!($cn$c)),
            then: Box::new(expr!($tn$t)),
            otherwise: None,
        }
    };
    (If{$cn:ident$c:tt, $tn:ident$t:tt, $on:ident$o:tt}) => {
        Expr::If {
            cond: Box::new(expr!($cn$c)),
            then: Box::new(expr!($tn$t)),
            otherwise: Some(Box::new(expr!($on$o))),
        }
    };
    (While{$cn:ident$c:tt, $bn:ident$b:tt}) => {
        Expr::While {
            cond: Box::new(expr!($cn$c)),
            body: Box::new(expr!($bn$b)),
        }
    };
}
