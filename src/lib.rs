use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser};

mod interpreter;
mod lexer;
mod parser;

pub fn ferric_main(source: &str) {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let parser = Parser::new(tokens);
    let stmts = parser.parse();

    let interpreter = Interpreter::new();
    interpreter.interpret(stmts);
}
