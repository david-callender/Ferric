use std::fs;
use std::io::stdout;

use ferric::{
    FerricError, interpreter::Interpreter, lexer::Lexer, loc::ProgramSrc, parser::Parser,
};

fn ferric_main(source: String) -> Result<(), FerricError> {
    let src = ProgramSrc::new(source);

    let stream = src.clone();
    let lexer = Lexer::new(stream.stream(), src.clone());

    let mut parser = Parser::new(lexer);
    let (expr, var_storage_size) = parser.parse()?;

    let mut output = stdout();

    let mut interpreter = Interpreter::new(&mut output, var_storage_size);
    interpreter.interpret(&expr)?;

    Ok(())
}

fn main() {
    let contents = fs::read_to_string("./executable/src.fe").expect("No such file located.");
    match ferric_main(contents) {
        Ok(()) => {}
        Err(err) => eprintln!("Ferric ran into an error:{err}"),
    }
}
