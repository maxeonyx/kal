use crate::{eval::Custom, eval::Eval, interpreter};
use interpreter::{Scope, Value};
use std::{collections::HashMap, fmt::Debug, rc::Rc};

pub fn scope(parent: Option<Rc<Scope>>) -> Rc<Scope> {
    Rc::new(Scope::with_bindings(parent, {
        let mut map = HashMap::new();

        use self::Intrinsic::*;
        use Value::Intrinsic;
        map.insert("symbol".into(), Intrinsic(Symbol));

        map
    }))
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Intrinsic {
    Symbol,
}

impl Intrinsic {
    pub fn code(self) -> Rc<dyn Eval> {
        use Intrinsic::*;
        match self {
            Symbol => symbol(),
        }
    }
    pub fn num_parameters(self) -> usize {
        use Intrinsic::*;
        match self {
            Symbol => 0,
        }
    }
}

fn symbol() -> Rc<dyn Eval> {
    Rc::new(Custom::new("IntrinsicSymbol", |int| {
        let symbol = int.gen_symbol();
        int.push_value(symbol);
    }))
}
