use tini::Ini;

use crate::file::{FUNCTION_DEFINITION_CHARACTER, VARIABLE_DEFINITION_CHARACTER};

const DEFAULT_FUNCTION_KEYWORD: &'static str = "function";
const DEFAULT_VARIABLE_KEYWORD: &'static str = "var";

#[derive(Debug)]
pub struct Config {
    pub function_keyword: Option<String>,
    pub variable_keyword: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            function_keyword: Some(DEFAULT_FUNCTION_KEYWORD.to_string()),
            variable_keyword: Some(DEFAULT_VARIABLE_KEYWORD.to_string()),
        }
    }
}

impl Config {
    pub fn new() -> Config {
        let mut config = Config::default();
        let config_file = Ini::from_file("transcript.ini").unwrap();

        if let Some(function) = config_file.get::<String>("keywords", "function") {
            config.function_keyword = Some(function.clone());
        }

        if let Some(variable) = config_file.get::<String>("keywords", "variable") {
            config.variable_keyword = Some(variable.clone());
        }

        config
    }
}

impl Config {
    pub fn match_function_keyword(&self, keyword: &str) -> bool {
        if keyword == FUNCTION_DEFINITION_CHARACTER {
            return true;
        }

        if keyword == DEFAULT_FUNCTION_KEYWORD {
            return true;
        }

        if let Some(ref config_keyword) = self.function_keyword {
            return keyword == config_keyword;
        }

        false
    }

    pub fn match_variable_keyword(&self, keyword: &str) -> bool {
        if keyword == VARIABLE_DEFINITION_CHARACTER {
            return true;
        }

        if keyword == DEFAULT_VARIABLE_KEYWORD {
            return true;
        }

        if let Some(ref config_keyword) = self.variable_keyword {
            return keyword == config_keyword;
        }

        false
    }
}
