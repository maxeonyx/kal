use super::{Interpreter, Value};
use crate::ast::Statement;
use std::{fmt::Debug, rc::Rc};

pub trait Eval: Debug {
    fn eval(&self, int: &mut Interpreter) -> Option<Value>;
    fn short_name(&self) -> &str;
}

pub trait IntoEval<T: ?Sized> {
    fn into_eval(self) -> Rc<T>;
}

impl<'a, T: Eval + 'a> IntoEval<dyn Eval + 'a> for Rc<T> {
    fn into_eval(self) -> Rc<dyn Eval + 'a> {
        self
    }
}

impl<'a, T: Eval + 'a> IntoEval<dyn Eval + 'a> for T {
    fn into_eval(self) -> Rc<dyn Eval + 'a> {
        Rc::new(self)
    }
}
