use std::fs;
use std::io::stdout;

use ferric::{
    interpreter::Interpreter,
    lexer::Lexer,
    loc::{Loc, ProgramSrc, Span},
    parser::Parser,
};

fn ferric_main(source: &str) {
    let s = ProgramSrc::new(source.to_owned());

    let span = Span::new(Loc::new(2, 1), Loc::new(7, 5));

    println!("{}", span.format(&s, "you did something wrong here"));

    return;

    let lexer = Lexer::new(source.bytes());

    let mut parser = Parser::new(lexer);
    let (expr, var_storage_size) = parser.parse();

    let mut output = stdout();

    let mut interpreter = Interpreter::new(&mut output, var_storage_size);
    interpreter.interpret(&expr);
}

fn main() {
    let contents = fs::read_to_string("./executable/src.fe").expect("No such file located.");
    ferric_main(&contents);
}
