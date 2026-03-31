pub mod interpreter;
pub mod lexer;
mod macros;
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::{interpreter::Interpreter, lexer::Lexer, parser::Parser};

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

    #[test]
    fn boolean() {
        assert_eq!(harness("print(true);"), "true\n");
        assert_eq!(harness("print(false);"), "false\n");
        assert_eq!(harness("if true  {print(1);} otherwise{print(0);};"), "1\n");
        assert_eq!(harness("if false {print(1);} otherwise{print(0);};"), "0\n");
    }
}
