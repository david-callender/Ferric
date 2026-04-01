use std::io::Write;

use ferric::{interpreter::Interpreter, lexer::Lexer, parser::Parser};
use wasm_bindgen::prelude::*;
use web_sys::HtmlPreElement;

struct JsWriter {
    output: HtmlPreElement,
}

impl Write for JsWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = str::from_utf8(buf).expect("Bytes written weren't utf8");
        let mut current_contents = self
            .output
            .text_content()
            .expect("output has no text_content");
        current_contents.push_str(s);
        self.output.set_text_content(Some(&current_contents));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[wasm_bindgen]
pub fn ferric(src: &str, output: HtmlPreElement) {
    output.set_text_content(Some(""));
    let lexer = Lexer::new(src.bytes());
    let (program, var_storage_size) = Parser::new(lexer).parse();
    let mut output = JsWriter { output };
    let mut interpreter = Interpreter::new(&mut output, var_storage_size);
    interpreter.interpret(&program);
}
