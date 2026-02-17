use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser};

mod interpreter;
mod lexer;
mod parser;

pub fn ferric_main(source: &str) {
    let lexer = Lexer::new(source.bytes());

    let mut parser = Parser::new(lexer);
    let expr = parser.parse();

    let interpreter = Interpreter::new();
    interpreter.interpret(expr);
}
