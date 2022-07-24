use super::{Token, TokenTypes};

pub struct Tokenizer {
    input: String,
    current_char: usize,
    current_line: usize,
    line_start: usize,

    token_start: usize,
    output: Vec<Token>,
}

impl Tokenizer {
    pub fn from_string(input: String) -> Self {
        Tokenizer {
            input,
            current_char: 0,
            token_start: 0,
            output: Vec::new(),

            current_line: 1,
            line_start: 0,
        }
    }
}

// Tokenization logic
impl Tokenizer {
    pub fn tokenize(mut self) -> Vec<Token> {
        while self.not_at_end() {
            self.scan_token();
        }

        self.output
    }

    fn scan_token(&mut self) {
        let current = self.advance();

        if current.is_none() {
            panic!("scan_token ran out of tokens too early!");
        }

        match current.unwrap() {
            '+' => self.add_token(TokenTypes::Plus),
            '-' => self.add_token(TokenTypes::Minus),
            '*' => self.add_token(TokenTypes::Star),
            '=' => self.add_token(TokenTypes::Equals),
            '/' => {
                if self.peek().unwrap() == '/' {
                    self.advance();
                    let comment_content = self.scan_comment().trim().to_string();
                    self.add_token(TokenTypes::Comment {
                        value: comment_content,
                    });
                } else {
                    self.add_token(TokenTypes::Slash);
                }
            }

            '(' => self.add_token(TokenTypes::OpenParen),
            ')' => self.add_token(TokenTypes::CloseParen),
            '{' => self.add_token(TokenTypes::OpenCurly),
            '}' => self.add_token(TokenTypes::CloseCurly),

            ';' => self.add_token(TokenTypes::Semi),
            ',' => self.add_token(TokenTypes::Comma),

            // Ignore whitespace
            ' ' | '\r' | '\t' => (),
            // TODO: Newline tokens should be tracked if they are not on lines
            // with a statement to ensure coherent spacing in the packed output
            '\n' => {
                self.current_line += 1;
                self.line_start = self.current_char;
                self.token_start = self.current_char; // Prevent overflows across multiple lines
            }

            '"' => self.scan_string(),
            '0'..='9' => self.scan_number(),
            current => self.scan_identifier(current),
        }
    }
}

/// Utility definitions
impl Tokenizer {
    fn advance(&mut self) -> Option<char> {
        self.current_char += 1;
        self.input.chars().nth(self.current_char - 1)
    }

    fn add_token(&mut self, token_type: TokenTypes) {
        if self.line_start > self.token_start {
            println!("Tokenizer: Token spanned across two lines");
        }

        self.output.push(Token {
            token_type,
            start: self.token_start - self.line_start,
            end: self.current_char - self.line_start,
            line: self.current_line,
            line_contents: self
                .input
                .split('\n')
                .nth(self.current_line - 1)
                .unwrap()
                .to_string(),
        });

        self.token_start = self.current_char;
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.current_char)
    }

    fn scan_comment(&mut self) -> String {
        let mut comment = String::new();

        while self.peek().unwrap() != '\n' {
            comment.push(self.advance().unwrap());
        }

        comment
    }

    fn scan_string(&mut self) {
        let mut current = self.advance();
        let mut string = String::new();

        // TODO: We might want to support escaping quotes
        // TODO: We might not want to include newline characters
        while current != Some('"') && current.is_some() {
            string.push(current.unwrap());
            current = self.advance();
        }

        self.add_token(TokenTypes::String { value: string });
    }

    fn scan_number(&mut self) {
        let mut current = self.advance();
        let mut number = String::new();

        while current.is_some() && current.unwrap().is_digit(10) {
            number.push(current.unwrap());
            current = self.advance();
        }

        self.add_token(TokenTypes::Number {
            value: number.parse().unwrap(),
        });
    }

    fn scan_identifier(&mut self, first: char) {
        let mut current = self.advance();
        let mut identifier = String::new();
        identifier.push(first);

        while current.is_some() && current.unwrap().is_alphanumeric() {
            identifier.push(current.unwrap());
            current = self.advance();
        }

        self.current_char -= 1;

        self.add_token(TokenTypes::Identifier { value: identifier });
    }

    fn not_at_end(&self) -> bool {
        self.input.chars().nth(self.current_char).is_some()
    }
}
