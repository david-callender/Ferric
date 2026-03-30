use std::io::stdout;

use ferric::{interpreter::Interpreter, lexer::Lexer, parser::Parser};

fn ferric_main(source: &str) {
    let lexer = Lexer::new(source.bytes());

    let mut parser = Parser::new(lexer);
    let (expr, var_storage_size) = parser.parse();

    let mut output = stdout();

    let mut interpreter = Interpreter::new(&mut output, var_storage_size);
    interpreter.interpret(&expr);
}

fn main() {
    let source = include_str!("src.txt");
    ferric_main(source);
}
