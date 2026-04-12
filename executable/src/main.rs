use std::fs;
use std::io::stdout;

use ferric::{
    interpreter::Interpreter,
    lexer::{Lexer, LexerError},
    loc::{Loc, ProgramSrc, Span},
    parser::Parser,
};

fn ferric_main(source: String) {
    let src = ProgramSrc::new(source);
    
    // let err = LexerError::NumLitLeadingDecimal(src.clone(), Span::new(Loc::new(1, 1), Loc::new(1, 1)));
    
    // eprintln!("{err}");
    // return;
    
    let stream = src.clone();
    let lexer = Lexer::new(stream.stream(), src.clone());
    
    let tokens = lexer.collect::<Vec<_>>();
    
    for l in tokens {
        println!("{}", l.span.format(&src, ""));
        println!();
    }
    
    return;
    

    let mut parser = Parser::new(lexer);
    let (expr, var_storage_size) = parser.parse();

    let mut output = stdout();

    let mut interpreter = Interpreter::new(&mut output, var_storage_size);
    interpreter.interpret(&expr);
}

fn main() {
    let contents = fs::read_to_string("./executable/src.fe").expect("No such file located.");
    ferric_main(contents);
}
