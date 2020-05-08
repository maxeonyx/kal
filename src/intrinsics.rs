use crate::{eval::Custom, eval::Eval, interpreter};
use interpreter::{Scope, Value};
use std::{collections::HashMap, fmt::Debug, rc::Rc};

pub fn intrinsic_scope(parent: Option<Rc<Scope>>) -> Rc<Scope> {
    Rc::new(Scope::with_bindings(parent, {
        let mut map = HashMap::new();
        map
    }))
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Intrinsic {}

impl Intrinsic {
    pub fn code(self) -> Rc<dyn Eval> {
        use Intrinsic::*;
        unimplemented!()
    }
    pub fn num_parameters(self) -> usize {
        use Intrinsic::*;
        unimplemented!()
    }
}
