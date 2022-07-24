use std::fmt::Display;

use crate::config::Config;

use super::{Token, TokenTypes};

type BAST = Box<AST>;

#[derive(Debug, Clone)]
pub enum AST {
    Block {
        statements: Vec<AST>,
    },
    VariableDefinition {
        name: String,
        value: BAST,
    },
    FunctionDefinition {
        name: String,
        params: Vec<Token>,
        body: BAST,
    },
    FunctionCall {
        name: String,
        args: Vec<AST>,
    },
    Assignment {
        name: String,
        value: BAST,
    },
    Comment {
        value: String,
    },

    // Expression symbols
    Term(BAST, TermSymbol, BAST),
    Factor(BAST, FactorSymbol, BAST),
    Unary(TermSymbol, BAST),
    Group(BAST),
    Number(f32),
    String(String),
    Identifier(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TermSymbol {
    Add,
    Sub,
}

impl Display for TermSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermSymbol::Add => write!(f, "+"),
            TermSymbol::Sub => write!(f, "-"),
        }
    }
}

/// The symbol used for factor eqns. Either * (Mul) or / (Div)
#[derive(Debug, Clone)]
pub enum FactorSymbol {
    Mul,
    Div,
}

type Tokens = Vec<Token>;

#[inline]
pub fn parse(tokens: &mut Tokens, config: &Config) -> AST {
    // Rust works best with pop and doesn't like you removing the first element
    // from a vec because it is slow. The solution is to reverse the vec so it
    // is faster
    tokens.reverse();

    parse_block(tokens, config)
}

pub fn parse_block(tokens: &mut Tokens, config: &Config) -> AST {
    AST::Block {
        statements: parse_block_internal(tokens, config),
    }
}

fn is_valid_body_token(token: &Token) -> bool {
    match &token.token_type {
        TokenTypes::Identifier { .. } | TokenTypes::Comment { .. } => true,
        _ => false,
    }
}

fn peek(tokens: &mut Tokens) -> Option<Token> {
    tokens.last().cloned()
}

fn parse_block_internal(tokens: &mut Tokens, config: &Config) -> Vec<AST> {
    let mut statements = Vec::new();

    while tokens.len() != 0 && is_valid_body_token(&tokens[tokens.len() - 1]) {
        let is_comment = match &tokens[tokens.len() - 1].token_type {
            TokenTypes::Comment { .. } => true,
            _ => false,
        };

        statements.push(*parse_statement(tokens, config));

        if !is_comment {
            // Expect semicolon
            let semicolon = tokens.pop().unwrap();
            if semicolon.token_type != TokenTypes::Semi {
                println!("Expected semicolon, got: {:?}", semicolon);
                panic!("Expected semicolon!");
            }
        }
    }

    statements
}

fn parse_statement(tokens: &mut Tokens, config: &Config) -> BAST {
    let token = tokens.pop().unwrap();

    match token.token_type {
        TokenTypes::Identifier { value: keyword } => {
            if config.match_function_keyword(&keyword)
                && peek(tokens).unwrap().token_type != TokenTypes::OpenParen
            {
                return parse_function_definition(tokens, config);
            }

            if config.match_variable_keyword(&keyword)
                && peek(tokens).unwrap().token_type.is_identifier()
            {
                return parse_variable_definition(tokens);
            }

            if peek(tokens).unwrap().token_type == TokenTypes::OpenParen {
                return parse_function_call(tokens, keyword);
            }

            if peek(tokens).unwrap().token_type == TokenTypes::Equals {
                return parse_assignment(tokens, keyword);
            }

            panic!("Unimplemented statement type");
        }
        TokenTypes::Comment { value } => Box::new(AST::Comment { value }),
        _ => panic!("Unexpected token: {:?}", token),
    }
}

fn parse_function_definition(tokens: &mut Tokens, config: &Config) -> BAST {
    let name = match tokens.pop().unwrap().token_type {
        TokenTypes::Identifier { value } => value,
        _ => panic!("Expected function name"),
    };

    let params: Vec<Token> = Vec::new();
    let mut token = tokens.pop().unwrap();

    if token.token_type != TokenTypes::OpenParen {
        panic!("Expected '('");
    }

    token = tokens.pop().unwrap();
    // TODO: add function parameter support

    if token.token_type != TokenTypes::CloseParen {
        panic!("Expected ')'");
    }

    if tokens.pop().unwrap().token_type != TokenTypes::OpenCurly {
        panic!("Expected '{{'");
    }

    let body = parse_block(tokens, config);

    let next_token = tokens.pop().unwrap();
    if next_token.token_type != TokenTypes::CloseCurly {
        panic!("Expected '}}'");
    }

    Box::new(AST::FunctionDefinition {
        name,
        params,
        body: Box::new(body),
    })
}

fn parse_variable_definition(tokens: &mut Tokens) -> BAST {
    let token = tokens.pop().unwrap();

    let name = match token.token_type {
        TokenTypes::Identifier { value } => value,
        _ => panic!("Expected variable name"),
    };

    if tokens.pop().unwrap().token_type != TokenTypes::Equals {
        panic!("Expected '='");
    }

    let value = parse_expression(tokens);

    Box::new(AST::VariableDefinition { name, value })
}

fn parse_function_call(tokens: &mut Tokens, name: String) -> BAST {
    let mut args = Vec::new();

    if tokens.pop().unwrap().token_type != TokenTypes::OpenParen {
        panic!("Expected '('");
    }

    let mut current_token = tokens[tokens.len() - 1].clone();

    if current_token.token_type == TokenTypes::CloseParen {
        tokens.pop();
    }

    while current_token.token_type != TokenTypes::CloseParen {
        let arg = *parse_expression(tokens);
        args.push(arg);

        current_token = tokens.pop().unwrap();

        // Remove a comma between args in a function call
        if current_token.token_type == TokenTypes::Comma {
            current_token = tokens.pop().unwrap();
        }
    }

    Box::new(AST::FunctionCall { name, args })
}

fn parse_assignment(tokens: &mut Tokens, keyword: String) -> BAST {
    let name = keyword;

    if tokens.pop().unwrap().token_type != TokenTypes::Equals {
        panic!("Expected '='");
    }

    let value = parse_expression(tokens);

    Box::new(AST::Assignment { name, value })
}

#[inline]
fn parse_expression(tokens: &mut Tokens) -> BAST {
    term(tokens)
}

/// Responsible for parsing basic addition and subtraction. Because it is at the
/// top of the stack, it has the lowest priority.
///
/// Based on the following rule:
/// ```ebnf
/// term ::= factor ['+' | '-' term]
/// ```
fn term(tokens: &mut Tokens) -> BAST {
    // We should execute factor first, as described by the grammar rule
    //
    // Note on mut: within rust, you have to explicitly tell the compiler that a
    // variable can be changed. This is done with the `mut` keyword. Here, `raw`
    // is mutable, but `left` is immutable
    let left = factor(tokens);

    let symbol = &tokens[tokens.len() - 1].token_type;

    // If `'+' | '-'` does not match, we should return the value generated by
    // factor.
    if !(*symbol == TokenTypes::Plus || *symbol == TokenTypes::Minus) {
        return left;
    }

    // Convert the string into the enum that is required by AST::Term.
    let operator = match tokens.pop().unwrap().token_type {
        TokenTypes::Plus => TermSymbol::Add,
        TokenTypes::Minus => TermSymbol::Sub,
        // We have already checked for '+' or '-' above, so we do not need to
        // handle the case here
        _ => unreachable!(),
    };

    // We want to execute term on the left.
    // Whilst we are doing it because the grammar says to do it, the grammar says
    // to do it because it allows for the chaining of multiple terms (e.g. 1+2+3)
    let right = term(tokens);

    // Return both the modified string (mainly modified by pop_first) and the
    // enum
    return
        // Box puts the variable on the heap. This allows for the contents of
        // Box (i.e. Expr) to not have a size that is known at compile time.
        Box::new(
            // This is the rust method of constructing a tuple based enum. It
            // may look a bit funky, but think of it as a class invocation and
            // you will be fine.
            AST::Term(left, operator, right),
        );
}

/// Responsible for handling multiplication and division. This function has a
/// higher priority than terms (addition and subtraction), but a lower priority
/// than unary (negations) or groups.
///
/// Based on the following rules:
/// ```ebnf
/// factor ::= unary ['/' | '*' factor]
/// ```
fn factor(tokens: &mut Tokens) -> BAST {
    // Jump down to unary. It has a higher priority, so should be parsed before
    // the rest of this function
    let left = unary(tokens);

    let symbol = &tokens[tokens.len() - 1].token_type;

    // Just return the unary expression if it is not multiplication or division.
    // Either something below this function has already parsed it, or it will
    // be parsed by the term function above
    if !(*symbol == TokenTypes::Star || *symbol == TokenTypes::Slash) {
        return left;
    }

    // Convert the first character of the string into an enum
    let operator = match tokens.pop().unwrap().token_type {
        TokenTypes::Star => FactorSymbol::Mul,
        TokenTypes::Slash => FactorSymbol::Div,
        // Because we already checked for other characters at the start, we know
        // it will either be * or /
        _ => unreachable!(),
    };

    // We want to repeat the factor to allow for chaining
    let right = factor(tokens);

    // Return the necessary values
    return Box::new(AST::Factor(left, operator, right));
}

/// Handles negated numbers. This function is also responsible for triggering
/// `group` and `number`
///
/// Based on the following rule:
/// ```ebnf
/// unary ::= ['+' | '-'] group | number | unary
/// ```
fn unary(tokens: &mut Tokens) -> BAST {
    // Send groups of to a separate functions to be handled. If it is pretended
    // by a -, it will be sent through unary anyway. The grammar is cleaner if
    // this inconsistency is ignored.
    if tokens[tokens.len() - 1].token_type == TokenTypes::OpenParen {
        return group(tokens);
    }

    let token = tokens.pop().unwrap().token_type;

    // If it doesn't start with a + or a -, we should send it through to the
    // number parser to get parsed
    if !(token == TokenTypes::Plus || token == TokenTypes::Minus) {
        return match token {
            TokenTypes::Number { value } => Box::new(AST::Number(value)),
            TokenTypes::String { value } => Box::new(AST::String(value)),
            TokenTypes::Identifier { value } => Box::new(AST::Identifier(value)),
            _ => {
                panic!("Expected number, string or identifier");
            }
        };
    }

    let symbol = match token {
        TokenTypes::Plus => TermSymbol::Add,
        TokenTypes::Minus => TermSymbol::Sub,
        // We have already checked for '+' or '-' above, so we do not need to
        // handle the case here
        _ => unreachable!(),
    };

    // Recursion time. Sends it back to handle the number, group and second
    // unary case.
    let num = unary(tokens);

    // Return all of th necessary values
    return Box::new(AST::Unary(symbol, num));
}

/// Handles everything within parenthesizes.
///
/// Based on the following rule:
/// ```ebnf
/// group ::= '(' expression ')'
/// ```
fn group(tokens: &mut Tokens) -> BAST {
    // Must start with an opening bracket
    if tokens.pop().unwrap().token_type != TokenTypes::OpenParen {
        panic!("Expected '('");
    }

    // Jump all of the way back up to the expression function.
    // See? I told you that the expression function would make the code more
    // readable latter
    let expr = parse_expression(tokens);

    // Check for closing bracket
    if tokens.pop().unwrap().token_type != TokenTypes::CloseParen {
        panic!("Expected ')'");
    }

    // Return necessary values
    return Box::new(AST::Group(expr));
}
