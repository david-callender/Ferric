use std::fs;
use std::io::stdout;

use ferric::{
    interpreter::Interpreter,
    lexer::Lexer,
    loc::{Loc, ProgramSrc},
    parser::Parser,
};

fn ferric_main(source: &str) {
    let s = ProgramSrc::new(source.to_owned());

    let loc = Loc::new(3, 0, 3);

    println!("{}", loc.format(&s));

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
