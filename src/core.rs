use crate::interpreter;
use interpreter::{symbols, symbols::error_codes, Key, Scope};
use std::{collections::HashMap, rc::Rc};

pub fn scope(parent: Option<Rc<Scope>>) -> Rc<Scope> {
    Rc::new(Scope::with_bindings(parent, {
        let mut map = HashMap::new();

        use interpreter::Value::*;
        map.insert("error".into(), Symbol(symbols::ERROR));
        map.insert("errors".into(), Object(Rc::new({
            let mut map = HashMap::new();
            map.insert(Key::Str("type_error_int".into()), Symbol(error_codes::TYPE_ERROR_INT));
            map.insert(Key::Str("error_loop".into()), Symbol(error_codes::ERROR_LOOP));
            map
        })));

        map
    }))
}