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

    #[test]
    fn arithmetic() {
         assert_eq!(harness("print(4 - 5);"), "-1\n");
         //assert_eq!(harness("print(-4);"), "-4\n");
         assert_eq!(harness("print(25 - 5 * 5);"), "0\n");
         assert_eq!(harness("print(50 / 2 - 5 * 5);"), "0\n");
         assert_eq!(harness("print(100 / (2 * 2) - 5 * 5);"), "0\n");
    }

    #[test]
    fn blocks() {
        assert_eq!(harness("{             };"), "");
        assert_eq!(harness("{print(4)};"), "4\n");
        assert_eq!(harness("{print(4)};"), "4\n");
    }

    #[test]
    fn compare() {
        assert_eq!(harness(""), "");
    }

    #[test]
    fn vars() {
        assert_eq!(harness("let a = 0; print(a); let b = 1; print(b); let c = 2; print(c); let d = 3; print(d);"), "0\n1\n2\n3\n");
        assert_eq!(harness("let x = 0; print(x); let x = 2; print(x);"), "0\n2\n");
        assert_eq!(harness("let x = 0; print(x); if x == 0 {let x = 2; print(x)}; print(x);"), "0\n2\n0\n");
        assert_eq!(harness("let x = 0; print(x); let x = 2; print(x);"), "0\n2\n");
    }
}


