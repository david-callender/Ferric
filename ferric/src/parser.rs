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
    VarSet {
        value: Box<Expr>,
        slot: usize,
    },
    Block(Vec<Expr>),
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        otherwise: Option<Box<Expr>>,
    },
    While {
        cond: Box<Expr>,
        body: Box<Expr>,
    },
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
    pub fn new<II: IntoIterator<IntoIter = I>>(stream: II) -> Self {
        Self {
            stream: stream.into_iter().peekable(),
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

    fn is_one_of<const N: usize>(&mut self, expected: [Token; N]) -> Option<Token> {
        if self.stream.peek().is_some_and(|tok| expected.contains(tok)) {
            return Some(self.stream.next().unwrap());
        }
        drop(expected);
        None
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
        } else if self.matches(Token::While) {
            self.parse_while()
        } else {
            self.parse_var_set()
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

    fn parse_while(&mut self) -> Expr {
        let cond = Box::new(self.parse_expr());
        self.consume(Token::OpenBracket, "Expected '{' after while");
        let body = Box::new(self.parse_block());
        Expr::While { cond, body }
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

    fn parse_var_set(&mut self) -> Expr {
        let left = self.parse_comparisons();

        if self.matches(Token::Eq) {
            let right = self.parse_comparisons();

            let Expr::VarGet { slot } = left else {
                panic!("Expected variable name to be an identifier");
            };

            return Expr::VarSet {
                value: Box::new(right),
                slot,
            };
        }
        left
    }

    fn parse_comparisons(&mut self) -> Expr {
        let left = self.parse_add_subtract();
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
            None => return left,
        };
        let right = self.parse_add_subtract();
        Expr::Binary {
            left: Box::new(left),
            operation,
            right: Box::new(right),
        }
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
            Token::True => Expr::Literal(RuntimeVal::Boolean(true)),
            Token::False => Expr::Literal(RuntimeVal::Boolean(false)),
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
    use crate::{expr, one_token, tokens};

    use super::*;

    #[test]
    fn num_literal() {
        let mut parser = Parser::new(tokens!(NumLit(4.0)));
        assert_eq!(parser.parse_expr(), expr!(NumLit(4.0)));
    }

    #[test]
    fn string_literal() {
        let mut parser = Parser::new(tokens!(StrLit("dingus")));
        assert_eq!(parser.parse_expr(), expr!(StrLit("dingus")));
    }

    #[test]
    fn parentheses() {
        let mut parser = Parser::new(tokens!(OpenParen, NumLit(4.0), CloseParen));
        assert_eq!(parser.parse_expr(), expr!(NumLit(4.0)));
    }

    #[test]
    fn add() {
        let mut parser = Parser::new(tokens!(NumLit(4.0), Plus, NumLit(5.0)));
        let target = expr!(Binary(NumLit(4.0), Add, NumLit(5.0)));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn subtract() {
        let mut parser = Parser::new(tokens!(NumLit(4.0), Minus, NumLit(5.0)));
        let target = expr!(Binary(NumLit(4.0), Subtract, NumLit(5.0)));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn multiply() {
        let mut parser = Parser::new(tokens!(NumLit(20.0), Star, NumLit(22.0)));
        let target = expr!(Binary(NumLit(20.0), Multiply, NumLit(22.0)));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn divide() {
        let mut parser = Parser::new(tokens!(NumLit(20.0), Slash, NumLit(22.0)));
        let target = expr!(Binary(NumLit(20.0), Divide, NumLit(22.0)));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn simple_funcall() {
        let mut parser = Parser::new(tokens!(Ident("my_func"), OpenParen, CloseParen));
        let target = expr!(Call(Ident("my_func"), []));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn complex_funcall() {
        let mut parser = Parser::new(tokens!(
            Ident("my_func".to_string()),
            OpenParen,
            NumLit(42.0),
            Comma,
            NumLit(88.0),
            CloseParen,
            OpenParen,
            StringLit("dingus".to_string()),
            CloseParen
        ));
        let target = expr!(Call(
            Call(Ident("my_func"), [NumLit(42.0), NumLit(88.0)]),
            [StrLit("dingus")]
        ));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn unary_minus() {
        let mut parser = Parser::new(tokens!(Minus, NumLit(6.0)));
        let target = expr!(Unary(Negate, NumLit(6.0)));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn unary_bitnot() {
        let mut parser = Parser::new(tokens!(Tilde, NumLit(6.0)));
        let target = expr!(Unary(BitNot, NumLit(6.0)));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn unary_and_minus() {
        let mut parser = Parser::new(tokens!(Minus, NumLit(6.0), Minus, NumLit(6.0)));
        let target = expr!(Binary(Unary(Negate, NumLit(6.0)), Subtract, NumLit(6.0)));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn unary_with_parens() {
        let mut parser = Parser::new(tokens!(
            Minus,
            OpenParen,
            NumLit(6.0),
            Plus,
            NumLit(6.0),
            CloseParen
        ));
        let target = expr!(Unary(Negate, Binary(NumLit(6.0), Add, NumLit(6.0))));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn block() {
        // empty
        let mut parser = Parser::new(tokens!(OpenBracket, CloseBracket, Semi));
        let target = Expr::Block(vec![]);
        assert_eq!(parser.parse_expr(), target);

        // one, return last
        let mut parser = Parser::new(tokens!(OpenBracket, NumLit(4.0), CloseBracket, Semi));
        let target = expr!(Block { NumLit(4.0) });
        assert_eq!(parser.parse_expr(), target);

        // one, don't return last
        let mut parser = Parser::new(tokens!(OpenBracket, NumLit(4.0), Semi, CloseBracket, Semi));
        let target = expr!(Block {
            NumLit(4.0),
            Null()
        });
        assert_eq!(parser.parse_expr(), target);

        // many, return last
        let mut parser = Parser::new(tokens!(
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
        assert_eq!(parser.parse_expr(), target);

        // many, don't return last
        let mut parser = Parser::new(tokens!(
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
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn if_expr() {
        // if
        let mut parser = Parser::new(tokens!(
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

        assert_eq!(parser.parse_expr(), target);

        // if otherwise
        let mut parser = Parser::new(tokens!(
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

        assert_eq!(parser.parse_expr(), target);

        // if otherwise-if otherwise
        let mut parser = Parser::new(tokens!(
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

        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn comparisons() {
        let mut parser = Parser::new(tokens!(NumLit(3.0), EqEq, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), Equal, NumLit(4.0)));
        assert_eq!(parser.parse_expr(), target);

        let mut parser = Parser::new(tokens!(NumLit(3.0), Greater, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), GreaterThan, NumLit(4.0)));
        assert_eq!(parser.parse_expr(), target);

        let mut parser = Parser::new(tokens!(NumLit(3.0), LessEq, NumLit(4.0)));
        let target = expr!(Binary(NumLit(3.0), LessEq, NumLit(4.0)));
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn comparison_order() {
        let mut parser = Parser::new(tokens!(
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
        assert_eq!(parser.parse_expr(), target);
    }

    #[test]
    fn while_expr() {
        let mut parser = Parser::new(tokens!(
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

        assert_eq!(parser.parse_expr(), target);
    }
}
