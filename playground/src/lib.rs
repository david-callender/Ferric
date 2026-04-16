use std::io::Write;

use chrono::Utc;
use ferric::{interpreter::Interpreter, lexer::Lexer, loc::ProgramSrc, parser::Parser};
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
pub fn init(output: HtmlPreElement) {
    std::panic::set_hook(Box::new(move |info| {
        let current = output.text_content().unwrap_or_default();
        output.set_text_content(Some(&format!("{current}Ferric ran into an error:\n{info}")));
    }));
}

#[wasm_bindgen]
pub fn ferric(src: &str, output: HtmlPreElement) {
    output.set_text_content(Some(""));
    let mut output = JsWriter { output };

    let src = ProgramSrc::new(src.to_string());

    let stream = src.clone();
    let lexer = Lexer::new(stream.stream(), src);

    let parser_start = Utc::now();
    let program = Parser::new(lexer).parse().unwrap();
    let parser_time = Utc::now() - parser_start;

    let interpreter_start = Utc::now();
    let mut interpreter = Interpreter::new(&mut output);
    interpreter.interpret(&program).unwrap();
    let interpreter_time = Utc::now() - interpreter_start;

    write!(
        output,
        "----------------------------------------\nparser took {}ms\ninterpreter took {}ms",
        parser_time.num_milliseconds(),
        interpreter_time.num_milliseconds(),
    )
    .expect("failed to write to output");
}
