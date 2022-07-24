use crate::utils::Config;

use super::{FUNCTION_DEFINITION_CHARACTER, VARIABLE_DEFINITION_CHARACTER};

pub fn unpack(source: String, config: &Config) -> String {
    source
        .replace(VARIABLE_DEFINITION_CHARACTER, &config.variable_keyword)
        .replace(FUNCTION_DEFINITION_CHARACTER, &config.function_keyword)
}
