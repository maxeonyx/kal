use super::{Interpreter, Value};
use std::fmt::Debug;

pub trait Eval: Debug {
    fn eval(&self, int: &mut Interpreter) -> Option<Value>;
    fn short_name(&self) -> &str;
}
