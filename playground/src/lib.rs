use std::io::Write;

use chrono::Utc;
use ferric::{interpreter::Interpreter, lexer::Lexer, parser::Parser};
use wasm_bindgen::prelude::*;
use web_sys::HtmlPreElement;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = performance)]
//     fn now() -> f64;
// }

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
pub fn init(output: HtmlPreElement) {
    std::panic::set_hook(Box::new(move |info| {
        output.set_text_content(Some(&format!("panic: {info}")));
    }));
}

#[wasm_bindgen]
pub fn ferric(src: &str, output: HtmlPreElement) {
    output.set_text_content(Some(""));
    let mut output = JsWriter { output };

    let lexer = Lexer::new(src.bytes());

    let parser_start = Utc::now();
    let (program, var_storage_size) = Parser::new(lexer).parse();
    let parser_time = Utc::now() - parser_start;

    let interpreter_start = Utc::now();
    let mut interpreter = Interpreter::new(&mut output, var_storage_size);
    interpreter.interpret(&program);
    let interpreter_time = Utc::now() - interpreter_start;

    write!(
        output,
        "----------------------------------------\nparser took {}ms\ninterpreter took {}ms",
        parser_time.num_milliseconds(),
        interpreter_time.num_milliseconds(),
    )
    .expect("failed to write to output");
}
