mod parse;
/// This module is responsible for translating a specified file into a tokenized
/// file and back.
mod tokenizer;

pub use parse::*;
pub use tokenizer::*;

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenTypes,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub line_contents: String,
}

#[derive(PartialEq, Clone, Debug)]
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

impl TokenTypes {
    pub fn is_identifier(&self) -> bool {
        match self {
            TokenTypes::Identifier { .. } => true,
            _ => false,
        }
    }
}
