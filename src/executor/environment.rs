use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{RustFunctionBody, Value};

pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn from_enclosing(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn assign(&mut self, name: &str, value: Value) {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return;
        }

        if let Some(ref mut enclosing) = self.enclosing {
            enclosing.borrow_mut().assign(name, value);
            return;
        }

        panic!("Undefined variable: {}", name);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            return Some(value.clone());
        }

        if let Some(ref env) = self.enclosing {
            return env.borrow().get(name);
        }

        None
    }

    pub fn contain(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub fn add_rust_function(&mut self, name: &str, args: Vec<String>, body: RustFunctionBody) {
        self.values
            .insert(name.to_string(), Value::RustFunction { args, body });
    }
}
