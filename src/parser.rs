use std::{collections::HashMap, iter::Peekable};

use crate::{interpreter::RuntimeVal, lexer::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(RuntimeVal),
    Ident(String),
    Binary {
        left: Box<Expr>,
        operation: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        operation: UnaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Decl {
        value: Box<Expr>,
        slot: usize,
    },
    VarGet {
        slot: usize,
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        otherwise: Option<Box<Expr>>,
    },
    Block(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEq,
    LessEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    BitNot,
}

pub struct Parser<I: Iterator<Item = Token>> {
    stream: Peekable<I>,
    next_index: usize,
    env: Vec<HashMap<String, usize>>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    // TODO: change stream to be IntoIter
    pub fn new(stream: I) -> Self {
        Self {
            stream: stream.peekable(),
            next_index: 0,
            env: vec![HashMap::new()], // global
        }
    }

    fn matches(&mut self, expected: Token) -> bool {
        if self.stream.peek() == Some(&expected) {
            let _ = self.stream.next().unwrap();
            drop(expected);
            true
        } else {
            false
        }
    }

    fn consume(&mut self, expected: Token, message: &str) -> Token {
        let token = self.stream.next().expect("expected token, got none");
        assert_eq!(token, expected, "{message}");
        drop(expected);
        token
    }

    fn consume_ident(&mut self, message: &str) -> String {
        let Some(Token::Ident(name)) = self.stream.next() else {
            panic!("{message}");
        };
        name
    }

    pub fn parse(&mut self) -> (Vec<Expr>, usize) {
        let mut exprs = vec![];
        while self.stream.peek().is_some() {
            exprs.push(self.parse_expr());
            self.consume(Token::Semi, "Expected ';' after expression");
        }
        (exprs, self.next_index)
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_keywords()
    }

    fn parse_keywords(&mut self) -> Expr {
        if self.matches(Token::Let) {
            self.parse_decl()
        } else if self.matches(Token::If) {
            self.parse_if()
        } else if self.matches(Token::OpenBracket) {
            self.parse_block()
        } else {
            self.parse_add_subtract()
        }
    }

    fn parse_decl(&mut self) -> Expr {
        let name = self.consume_ident("Expected variable name after let");
        self.consume(Token::Eq, "Expected '=' after let");
        let init = self.parse_expr();
        self.env
            .last_mut()
            .expect("no global env")
            .insert(name, self.next_index);
        let expr = Expr::Decl {
            value: Box::new(init),
            slot: self.next_index,
        };
        self.next_index += 1;
        expr
    }

    fn parse_if(&mut self) -> Expr {
        let cond = self.parse_expr();
        self.consume(Token::OpenBracket, "Expected '{' after if condition");

        let then = self.parse_block();

        let otherwise = if self.matches(Token::Otherwise) {
            let otherwise = if self.matches(Token::OpenBracket) {
                self.parse_block()
            } else if self.matches(Token::If) {
                self.parse_if()
            } else {
                panic!("Expected '{{' or 'if' after 'otherwise'");
            };

            Some(Box::new(otherwise))
        } else {
            None
        };

        Expr::If {
            cond: Box::new(cond),
            then: Box::new(then),
            otherwise,
        }
    }

    // assumes the leading Token::OpenBracket has already been consumed.
    fn parse_block(&mut self) -> Expr {
        if self.matches(Token::CloseBracket) {
            return Expr::Block(vec![]);
        }

        // each block creates its own scope, so add a blank scope to the
        // environment stack.
        self.env.push(HashMap::new());

        let mut exprs = vec![self.parse_expr()];
        while self.matches(Token::Semi) {
            if self.matches(Token::CloseBracket) {
                exprs.push(Expr::Literal(RuntimeVal::Null));
                self.env.pop().expect("misaligned environment stack");
                return Expr::Block(exprs);
            }
            exprs.push(self.parse_expr());
        }
        self.consume(
            Token::CloseBracket,
            "Expected '}' after block. Check for a missing semicolon on the previous line",
        );
        self.env.pop().expect("misaligned environment stack");
        Expr::Block(exprs)
    }

    fn parse_add_subtract(&mut self) -> Expr {
        let left = self.parse_multiply_divide();

        if self.matches(Token::Plus) {
            let right = self.parse_multiply_divide();
            return Expr::Binary {
                left: Box::new(left),
                operation: BinaryOp::Add,
                right: Box::new(right),
            };
        } else if self.matches(Token::Minus) {
            let right = self.parse_multiply_divide();
            return Expr::Binary {
                left: Box::new(left),
                operation: BinaryOp::Subtract,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_multiply_divide(&mut self) -> Expr {
        let left = self.parse_unary_op();
        if self.matches(Token::Star) {
            let right = self.parse_unary_op();
            return Expr::Binary {
                left: Box::new(left),
                operation: BinaryOp::Multiply,
                right: Box::new(right),
            };
        } else if self.matches(Token::Slash) {
            let right = self.parse_unary_op();
            return Expr::Binary {
                left: Box::new(left),
                operation: BinaryOp::Divide,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_unary_op(&mut self) -> Expr {
        if self.matches(Token::Minus) {
            let right = self.parse_func_call();
            return Expr::Unary {
                operation: UnaryOp::Negate,
                right: Box::new(right),
            };
        } else if self.matches(Token::Tilde) {
            let right = self.parse_func_call();
            return Expr::Unary {
                operation: UnaryOp::BitNot,
                right: Box::new(right),
            };
        }
        self.parse_func_call()
    }

    // consume_args also consumes the closing paren of the
    // arguments list, but assumes that the opening paren has
    // already been parsed.
    fn consume_args(&mut self) -> Vec<Expr> {
        let mut args_vec = vec![];

        if !self.matches(Token::CloseParen) {
            args_vec.push(self.parse_expr());
            while self.matches(Token::Comma) {
                args_vec.push(self.parse_expr());
            }
            self.consume(
                Token::CloseParen,
                "Unclosed function call parentheses or missing comma",
            );
        }

        args_vec
    }

    fn parse_func_call(&mut self) -> Expr {
        let mut func_call = self.parse_basic();

        while self.matches(Token::OpenParen) {
            let args_list = self.consume_args();
            func_call = Expr::Call {
                callee: Box::new(func_call),
                args: args_list,
            };
        }

        func_call
    }

    fn find_var(&self, name: &String) -> Option<usize> {
        for level in self.env.iter().rev() {
            if let Some(index) = level.get(name) {
                return Some(*index);
            }
        }
        None
    }

    fn parse_basic(&mut self) -> Expr {
        let token = self.stream.next().expect("expected basic token, got none");
        match token {
            Token::OpenParen => self.parse_paren(),
            Token::StringLit(string) => Expr::Literal(RuntimeVal::String(string)),
            Token::NumLit(number) => Expr::Literal(RuntimeVal::Number(number)),
            Token::Ident(identifier) => self
                .find_var(&identifier)
                .map(|slot| Expr::VarGet { slot })
                .unwrap_or(Expr::Ident(identifier)),
            _ => panic!("expected basic token, got non-basic token {token}"),
        }
    }

    // parse_paren assumes that the initial OpenParen token has already
    // been consumed.
    fn parse_paren(&mut self) -> Expr {
        let inner_expr = self.parse_expr();
        self.consume(Token::CloseParen, "unclosed paren block");
        inner_expr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_literal() {
        let mut parser = Parser::new([Token::NumLit(4.0)].into_iter());
        assert_eq!(parser.parse_expr(), Expr::Literal(RuntimeVal::Number(4.0)));
    }

    #[test]
    fn test_string_literal() {
        let mut parser = Parser::new([Token::StringLit("dingus".to_string())].into_iter());
        assert_eq!(
            parser.parse_expr(),
            Expr::Literal(RuntimeVal::String("dingus".to_string()))
        );
    }

    #[test]
    fn test_parentheses() {
        let mut parser =
            Parser::new([Token::OpenParen, Token::NumLit(4.0), Token::CloseParen].into_iter());
        assert_eq!(parser.parse_expr(), Expr::Literal(RuntimeVal::Number(4.0)));
    }

    #[test]
    fn test_add() {
        let mut parser =
            Parser::new([Token::NumLit(4.0), Token::Plus, Token::NumLit(5.0)].into_iter());
        let target = Expr::Binary {
            left: Box::new(Expr::Literal(RuntimeVal::Number(4.0))),
            operation: BinaryOp::Add,
            right: Box::new(Expr::Literal(RuntimeVal::Number(5.0))),
        };
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_subtract() {
        let mut parser =
            Parser::new([Token::NumLit(4.0), Token::Minus, Token::NumLit(5.0)].into_iter());
        let target = Expr::Binary {
            left: Box::new(Expr::Literal(RuntimeVal::Number(4.0))),
            operation: BinaryOp::Subtract,
            right: Box::new(Expr::Literal(RuntimeVal::Number(5.0))),
        };
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_multiply() {
        let mut parser =
            Parser::new([Token::NumLit(20.0), Token::Star, Token::NumLit(22.0)].into_iter());
        let target = Expr::Binary {
            left: Box::new(Expr::Literal(RuntimeVal::Number(20.0))),
            operation: BinaryOp::Multiply,
            right: Box::new(Expr::Literal(RuntimeVal::Number(22.0))),
        };
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_divide() {
        let mut parser =
            Parser::new([Token::NumLit(20.0), Token::Slash, Token::NumLit(22.0)].into_iter());
        let target = Expr::Binary {
            left: Box::new(Expr::Literal(RuntimeVal::Number(20.0))),
            operation: BinaryOp::Divide,
            right: Box::new(Expr::Literal(RuntimeVal::Number(22.0))),
        };
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_simple_funcall() {
        let mut parser = Parser::new(
            [
                Token::Ident("my_func".to_string()),
                Token::OpenParen,
                Token::CloseParen,
            ]
            .into_iter(),
        );

        let target = Expr::Call {
            callee: Box::new(Expr::Ident("my_func".to_string())),
            args: vec![],
        };

        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_complex_funcall() {
        let mut parser = Parser::new(
            [
                Token::Ident("my_func".to_string()),
                Token::OpenParen,
                Token::NumLit(42.0),
                Token::Comma,
                Token::NumLit(88.0),
                Token::CloseParen,
                Token::OpenParen,
                Token::StringLit("dingus".to_string()),
                Token::CloseParen,
            ]
            .into_iter(),
        );

        let target = Expr::Call {
            callee: Box::new(Expr::Call {
                callee: Box::new(Expr::Ident("my_func".to_string())),
                args: vec![
                    Expr::Literal(RuntimeVal::Number(42.0)),
                    Expr::Literal(RuntimeVal::Number(88.0)),
                ],
            }),
            args: vec![Expr::Literal(RuntimeVal::String("dingus".to_string()))],
        };

        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_unary_minus() {
        let mut parser = Parser::new([Token::Minus, Token::NumLit(6.0)].into_iter());

        let target = Expr::Unary {
            operation: UnaryOp::Negate,
            right: Box::new(Expr::Literal(RuntimeVal::Number(6.0))),
        };

        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_unary_bitnot() {
        let mut parser = Parser::new([Token::Tilde, Token::NumLit(6.0)].into_iter());

        let target = Expr::Unary {
            operation: UnaryOp::BitNot,
            right: Box::new(Expr::Literal(RuntimeVal::Number(6.0))),
        };

        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_unary_and_minus() {
        let mut parser = Parser::new(
            [
                Token::Minus,
                Token::NumLit(6.0),
                Token::Minus,
                Token::NumLit(6.0),
            ]
            .into_iter(),
        );

        let target = Expr::Binary {
            left: Box::new(Expr::Unary {
                operation: UnaryOp::Negate,
                right: Box::new(Expr::Literal(RuntimeVal::Number(6.0))),
            }),
            operation: BinaryOp::Subtract,
            right: Box::new(Expr::Literal(RuntimeVal::Number(6.0))),
        };

        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_unary_with_parens() {
        let mut parser = Parser::new(
            [
                Token::Minus,
                Token::OpenParen,
                Token::NumLit(6.0),
                Token::Plus,
                Token::NumLit(6.0),
                Token::CloseParen,
            ]
            .into_iter(),
        );

        let target = Expr::Unary {
            operation: UnaryOp::Negate,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal(RuntimeVal::Number(6.0))),
                operation: BinaryOp::Add,
                right: Box::new(Expr::Literal(RuntimeVal::Number(6.0))),
            }),
        };

        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn test_block() {
        // empty
        let parsed =
            Parser::new([Token::OpenBracket, Token::CloseBracket, Token::Semi].into_iter())
                .parse_expr();

        let target = Expr::Block(vec![]);

        assert_eq!(parsed, target);

        // one, return last
        let parsed = Parser::new(
            [
                Token::OpenBracket,
                Token::NumLit(4.0),
                Token::CloseBracket,
                Token::Semi,
            ]
            .into_iter(),
        )
        .parse_expr();

        let target = Expr::Block(vec![Expr::Literal(RuntimeVal::Number(4.0))]);

        assert_eq!(parsed, target);

        // one, don't return last
        let parsed = Parser::new(
            [
                Token::OpenBracket,
                Token::NumLit(4.0),
                Token::Semi,
                Token::CloseBracket,
                Token::Semi,
            ]
            .into_iter(),
        )
        .parse_expr();

        let target = Expr::Block(vec![
            Expr::Literal(RuntimeVal::Number(4.0)),
            Expr::Literal(RuntimeVal::Null),
        ]);

        assert_eq!(parsed, target);

        // many, return last
        let parsed = Parser::new(
            [
                Token::OpenBracket,
                Token::NumLit(4.0),
                Token::Semi,
                Token::NumLit(5.0),
                Token::CloseBracket,
                Token::Semi,
            ]
            .into_iter(),
        )
        .parse_expr();

        let target = Expr::Block(vec![
            Expr::Literal(RuntimeVal::Number(4.0)),
            Expr::Literal(RuntimeVal::Number(5.0)),
        ]);

        assert_eq!(parsed, target);

        // many, don't return last
        let parsed = Parser::new(
            [
                Token::OpenBracket,
                Token::NumLit(4.0),
                Token::Semi,
                Token::NumLit(5.0),
                Token::Semi,
                Token::CloseBracket,
                Token::Semi,
            ]
            .into_iter(),
        )
        .parse_expr();

        let target = Expr::Block(vec![
            Expr::Literal(RuntimeVal::Number(4.0)),
            Expr::Literal(RuntimeVal::Number(5.0)),
            Expr::Literal(RuntimeVal::Null),
        ]);

        assert_eq!(parsed, target);
    }

    #[test]
    fn test_if() {
        // if
        let parsed = Parser::new(
            [
                Token::If,
                Token::NumLit(1.0),
                Token::OpenBracket,
                Token::NumLit(4.0),
                Token::CloseBracket,
                Token::Semi,
            ]
            .into_iter(),
        )
        .parse_expr();

        let target = Expr::If {
            cond: Box::new(Expr::Literal(RuntimeVal::Number(1.0))),
            then: Box::new(Expr::Block(vec![Expr::Literal(RuntimeVal::Number(4.0))])),
            otherwise: None,
        };

        assert_eq!(parsed, target);

        // if otherwise
        let parsed = Parser::new(
            [
                Token::If,
                Token::NumLit(1.0),
                Token::OpenBracket,
                Token::NumLit(4.0),
                Token::CloseBracket,
                Token::Otherwise,
                Token::OpenBracket,
                Token::NumLit(5.0),
                Token::CloseBracket,
                Token::Semi,
            ]
            .into_iter(),
        )
        .parse_expr();

        let target = Expr::If {
            cond: Box::new(Expr::Literal(RuntimeVal::Number(1.0))),
            then: Box::new(Expr::Block(vec![Expr::Literal(RuntimeVal::Number(4.0))])),
            otherwise: Some(Box::new(Expr::Block(vec![Expr::Literal(
                RuntimeVal::Number(5.0),
            )]))),
        };

        assert_eq!(parsed, target);

        // if otherwise-if otherwise
        let parsed = Parser::new(
            [
                Token::If,
                Token::NumLit(1.0),
                Token::OpenBracket,
                Token::NumLit(4.0),
                Token::CloseBracket,
                Token::Otherwise,
                Token::If,
                Token::NumLit(2.0),
                Token::OpenBracket,
                Token::NumLit(5.0),
                Token::CloseBracket,
                Token::Otherwise,
                Token::OpenBracket,
                Token::NumLit(6.0),
                Token::CloseBracket,
                Token::Semi,
            ]
            .into_iter(),
        )
        .parse_expr();

        let target = Expr::If {
            cond: Box::new(Expr::Literal(RuntimeVal::Number(1.0))),
            then: Box::new(Expr::Block(vec![Expr::Literal(RuntimeVal::Number(4.0))])),
            otherwise: Some(Box::new(Expr::If {
                cond: Box::new(Expr::Literal(RuntimeVal::Number(2.0))),
                then: Box::new(Expr::Block(vec![Expr::Literal(RuntimeVal::Number(5.0))])),
                otherwise: Some(Box::new(Expr::Block(vec![Expr::Literal(
                    RuntimeVal::Number(6.0),
                )]))),
            })),
        };

        assert_eq!(parsed, target);
    }
}
