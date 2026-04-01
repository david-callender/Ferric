use std::io::Write;

use ferric::{interpreter::Interpreter, lexer::Lexer, parser::Parser};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(module = "/output.js")]
extern "C" {
    fn addToOutput(contents: &str);
}

struct JsWriter;

impl Write for JsWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = str::from_utf8(buf).expect("Bytes written weren't utf8");
        addToOutput(s);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[wasm_bindgen]
pub fn ferric(src: &str) {
    let lexer = Lexer::new(src.bytes());
    let (program, var_storage_size) = Parser::new(lexer).parse();
    let mut output = JsWriter;
    let mut interpreter = Interpreter::new(&mut output, var_storage_size);
    interpreter.interpret(&program);
}