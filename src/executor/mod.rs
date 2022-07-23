use core::panic;
use std::{
    cell::{RefCell, RefMut},
    fmt::Display,
    rc::Rc,
};

use crate::translate::{TermSymbol, TokenTypes, AST};

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
        body: Vec<AST>,
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
            Value::Function { args, body } => write!(f, "fn({}) {{ ... }}", args.join(", ")),
            Value::RustFunction { args, body } => {
                write!(f, "fn({}) {{ [Binary Code] }}", args.join(", "))
            }
        }
    }
}

pub fn execute(code: &AST, env: Rc<RefCell<Environment>>) -> Value {
    match code {
        AST::Program { statements } => {
            let mut results = Vec::new();
            for statement in statements {
                results.push(execute(statement, env.clone()));
            }

            Value::List(results)
        }
        AST::VariableDefinition { name, value } => {
            let value = execute(value, env.clone());
            env.borrow_mut().define(name, value);

            Value::Option(None)
        }
        AST::FunctionDefinition { name, params, body } => {
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

                    let mut val = Value::Option(None);
                    for statement in body {
                        val = execute(&statement, enclosing_environment.clone());
                    }

                    val
                }
                Value::RustFunction { args, body } => {
                    let mut enclosing_environment = Environment::from_enclosing(env.clone());

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
    }
}
