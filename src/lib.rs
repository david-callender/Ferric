use std::io::stdout;

use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser};

mod interpreter;
mod lexer;
mod parser;

pub fn ferric_main(source: &str) {
    let lexer = Lexer::new(source.bytes());

    let mut parser = Parser::new(lexer);
    let (expr, var_storage_size) = parser.parse();
    
    println!("{expr:?}");

    let mut output = stdout();

    let mut interpreter = Interpreter::new(&mut output, var_storage_size);
    // interpreter.interpret(&expr);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness(src: &str) -> String {
        let (expr, var_storage_size) = Parser::new(Lexer::new(src.bytes())).parse();

        let mut output = vec![];

        Interpreter::new(&mut output, var_storage_size).interpret(&expr);

        String::from_utf8(output).expect("Program outputted invalid utf8")
    }

    #[test]
    fn basic() {
        assert_eq!(harness("print(4);"), "4\n");
        assert_eq!(harness("print(4 + 5);"), "9\n");
        assert_eq!(harness("print(4 * 5);"), "20\n");
        assert_eq!(harness("print(7 + 4 * 5);"), "27\n");
        assert_eq!(harness("print((7 + 4) * 5);"), "55\n");
    }
}
