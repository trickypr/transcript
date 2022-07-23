mod parse;
/// This module is responsible for translating a specified file into a tokenized
/// file and back.
mod tokenizer;

pub use parse::*;
pub use tokenizer::*;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenTypes,
    start: usize,
    end: usize,
    line: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenTypes {
    Comment { value: String },
    Identifier { value: String },
    String { value: String },
    Number { value: f32 },

    // Required tokens. These are already a part of unicode so we can ignore
    // them
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
    Comma,

    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,

    Semi,
}
