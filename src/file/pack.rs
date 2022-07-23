use crate::translate::{TokenTypes, AST};

use super::{FUNCTION_DEFINITION_CHARACTER, VARIABLE_DEFINITION_CHARACTER};

pub fn pack(ast: &AST) -> String {
    let mut output = String::new();

    match ast {
        AST::Block { statements } => {
            for statement in statements {
                output.push_str(&pack(statement));
                output.push('\n');
            }
        }
        AST::VariableDefinition { name, value } => {
            output.push_str(&format!(
                "{} {} = {}",
                VARIABLE_DEFINITION_CHARACTER,
                name,
                pack(value)
            ));
        }
        AST::FunctionDefinition { name, params, body } => {
            output.push_str(&format!(
                "{} {}({}) {{\n{}\n}}",
                FUNCTION_DEFINITION_CHARACTER,
                name,
                params
                    .iter()
                    .map(|token| match &token.token_type {
                        TokenTypes::Identifier { value } => value.to_string(),
                        _ => panic!("Function parameters must be identifiers!"),
                    })
                    .collect::<Vec<String>>()
                    .join(", "),
                pack(body)
                    .split("\n")
                    .map(|line| format!("\t{}", line))
                    .collect::<Vec<String>>()
                    .join("\n")
            ));
        }
        AST::FunctionCall { name, args } => {
            output.push_str(&format!(
                "{}({})",
                name,
                args.iter()
                    .map(|ast| pack(ast))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }
        AST::Term(left, term, right) => {
            output.push_str(&format!("{} {} {}", pack(left), term, pack(right)));
        }
        AST::Factor(_, _, _) => todo!(),
        AST::Unary(_, _) => todo!(),
        AST::Group(_) => todo!(),
        AST::Number(_) => todo!(),
        AST::String(value) => output.push_str(&format!("\"{}\"", value)),
        AST::Identifier(name) => output.push_str(&name),
    }

    output
}
