use super::{Interpreter, Value};
use std::{fmt::Debug, rc::Rc};

pub trait Eval: Debug {
    fn eval(self: Rc<Self>, int: &mut Interpreter) -> Option<Value>;
    fn short_name(&self) -> &str;
}

pub trait IntoEval<T: ?Sized> {
    fn into_eval(self: Rc<Self>) -> Rc<T>;
}

impl<'a, T: Eval + 'a> IntoEval<dyn Eval + 'a> for T {
    fn into_eval(self: Rc<Self>) -> Rc<dyn Eval + 'a> {
        self
    }
}

pub trait UnimplementedEval: Debug {
    fn short_name(&self) -> &str;
}

impl<T: UnimplementedEval> Eval for T {
    fn eval(self: Rc<Self>, _: &mut Interpreter) -> Option<Value> {
        unimplemented!("unimplemented -- {} -- unimplemented", self.short_name())
    }
    fn short_name(&self) -> &str {
        self.short_name()
    }
}
