use core::panic;
use std::{
    cell::{RefCell, RefMut},
    fmt::Display,
    rc::Rc,
};

use crate::{
    file::{FUNCTION_DEFINITION_CHARACTER, VARIABLE_DEFINITION_CHARACTER},
    translate::{TermSymbol, Token, TokenTypes, AST},
    utils::warn_token,
};

pub use self::environment::Environment;

mod environment;

type RustFunctionBody = fn(Vec<Value>, RefMut<'_, Environment>) -> Value;

#[derive(Clone)]
pub enum Value {
    Number(f32),
    String(String),
    Option(Option<Box<Value>>),
    List(Vec<Value>),
    Function {
        args: Vec<String>,
        body: Box<AST>,
    },
    RustFunction {
        args: Vec<String>,
        body: RustFunctionBody,
    },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Option(o) => write!(f, "{}", o.as_ref().unwrap()),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Function { args, body: _ } => write!(f, "fn({}) {{ ... }}", args.join(", ")),
            Value::RustFunction { args, body: _ } => {
                write!(f, "fn({}) {{ [Binary Code] }}", args.join(", "))
            }
        }
    }
}

fn warn_about_text_tokens(token: &Token) {
    // We want to provide a warning to the user if they are directly
    // running a script to recommend that they pack it. Maybe in the
    // future this will become a hard error.
    warn_token(token, "Running scripts containing text-based keywords is not recommended. You should pack your scripts instead");
}

pub fn execute(code: &AST, env: Rc<RefCell<Environment>>) -> Value {
    match code {
        AST::Block { statements } => {
            let mut result = Value::Option(None);

            for statement in statements {
                result = execute(statement, env.clone());
            }

            result
        }
        AST::VariableDefinition {
            name,
            value,
            keyword_token,
        } => {
            if let TokenTypes::Identifier { value } = &keyword_token.token_type {
                if value != VARIABLE_DEFINITION_CHARACTER {
                    warn_about_text_tokens(&keyword_token);
                }
            }

            let value = execute(value, env.clone());
            env.borrow_mut().define(name, value);

            Value::Option(None)
        }
        AST::FunctionDefinition {
            name,
            params,
            body,
            keyword_token,
        } => {
            if let TokenTypes::Identifier { value } = &keyword_token.token_type {
                if value != FUNCTION_DEFINITION_CHARACTER {
                    warn_about_text_tokens(&keyword_token);
                }
            }

            let function = Value::Function {
                args: params
                    .iter()
                    .map(|token| match &token.token_type {
                        TokenTypes::Identifier { value } => value.to_string(),
                        _ => panic!("Function parameters must be identifiers!"),
                    })
                    .collect(),
                body: body.clone(),
            };

            env.borrow_mut().define(name, function);

            Value::Option(None)
        }
        AST::FunctionCall { name, args } => {
            // The return value is the last statement in the function
            let function = env
                .borrow()
                .get(name)
                .expect(&format!("Undefined function: {}", name));
            let call_args = args;

            match function {
                Value::Function { args, body } => {
                    let function_args = args;

                    let mut enclosing_environment = Environment::from_enclosing(env.clone());

                    for (i, arg) in call_args.iter().enumerate() {
                        let arg = execute(arg, env.clone());
                        enclosing_environment.assign(&function_args[i], arg);
                    }

                    let enclosing_environment = enclosing_environment.contain();

                    execute(&*body, enclosing_environment)
                }
                Value::RustFunction { args, body } => {
                    let enclosing_environment = Environment::from_enclosing(env.clone());

                    let call_args = call_args
                        .iter()
                        .enumerate()
                        .map(|(index, arg)| {
                            let arg = execute(arg, env.clone());
                            if index > args.len() {
                                panic!("Too many arguments for function!");
                            }
                            arg
                        })
                        .collect();

                    let enclosing_environment = enclosing_environment.contain();

                    body(call_args, enclosing_environment.borrow_mut())
                }
                _ => panic!("Cannot call non-function!"),
            }
        }
        AST::Assignment { name, value } => {
            let value = execute(value, env.clone());
            env.borrow_mut().assign(name, value);

            Value::Option(None)
        }
        AST::Term(left, op, right) => {
            let left = execute(left, env.clone());
            let right = execute(right, env.clone());

            match op {
                TermSymbol::Add => {
                    if let (Value::Number(l), Value::Number(r)) = (&left, &right) {
                        Value::Number(l + r)
                    } else if let (Value::String(l), Value::String(r)) = (left, right) {
                        Value::String(l + &r)
                    } else {
                        panic!("Cannot add non-numbers!")
                    }
                }
                TermSymbol::Sub => {
                    if let (Value::Number(l), Value::Number(r)) = (left, right) {
                        Value::Number(l - r)
                    } else {
                        panic!("Cannot subtract non-numbers!")
                    }
                }
            }
        }
        AST::Factor(_, _, _) => todo!(),
        AST::Unary(term, ast) => {
            let value = execute(ast, env.clone());

            Value::Number(match value {
                Value::Number(n) => match term {
                    TermSymbol::Add => n,
                    TermSymbol::Sub => -n,
                },
                _ => panic!("Cannot apply unary operator to non-number!"),
            })
        }
        AST::Group(_) => todo!(),
        AST::Number(_) => todo!(),
        AST::String(string) => Value::String(string.to_string()),
        AST::Identifier(name) => env
            .borrow()
            .get(name)
            .expect(&format!("Undefined variable: {}", name)),
        AST::Comment { value: _ } => Value::Option(None),
    }
}
