use super::{Context, Value};
use std::fmt::Debug;

pub trait Eval: Debug {
    fn eval(&self, ctx: &mut Context) -> Option<Value>;
}
