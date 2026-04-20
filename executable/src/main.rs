use std::io::stdout;
use std::{fs, rc::Rc};

use ferric::{
    FerricError, interpreter::Interpreter, lexer::Lexer, loc::ProgramSrcInner, parser::Parser,
};

fn ferric_main(source: String) -> Result<(), FerricError> {
    let src = Rc::new(ProgramSrcInner::new(source));

    let stream = src.clone();
    let lexer = Lexer::new(stream.stream(), src.clone());

    let mut parser = Parser::new(lexer, src.clone());
    let expr = parser.parse()?;

    let mut output = stdout();

    let mut interpreter = Interpreter::new(&mut output);
    interpreter.interpret(&expr)?;

    Ok(())
}

fn main() {
    let contents = fs::read_to_string("./executable/src.fe").expect("No such file located.");
    match ferric_main(contents) {
        Ok(()) => {}
        Err(err) => eprintln!("Ferric ran into an error:\n{err}"),
    }
}
