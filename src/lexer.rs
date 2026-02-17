use std::iter::Peekable;

pub enum Token {}

pub struct Lexer<I: Iterator<Item = u8>> {
    stream: Peekable<I>
}

impl<I: Iterator<Item = u8>> Iterator for Lexer<I> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<I: Iterator<Item = u8>> Lexer<I> {
    pub fn new(stream: I) -> Self {
        Self {
            stream: stream.peekable()
        }
    }
}
